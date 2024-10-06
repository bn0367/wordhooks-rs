mod commands;

extern crate dotenv;

use std::collections::HashSet;
use std::env;
use dotenv::dotenv;
use poise::{Framework, FrameworkOptions};
use poise::serenity_prelude::{ClientBuilder, CreateMessage, FullEvent, GatewayIntents, Mentionable, Message, UserId};
use sqlx::{Pool, Sqlite};
use tokio::sync::OnceCell;

static DEBUG: bool = true;


type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {}

static DB: OnceCell<Pool<Sqlite>> = OnceCell::const_new();


#[tokio::main]
async fn main()
{
    dotenv().ok();

    DB.set(sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(env::var("DATABASE_PATH").unwrap())
                .create_if_missing(true),
        )
        .await
        .expect("Couldn't connect to database")).expect("Could not set database (?)");

    sqlx::query!("CREATE TABLE IF NOT EXISTS hooks (user_id INTEGER, guild_id INTEGER, hook TEXT);")
        .execute(DB.get().unwrap())
        .await
        .unwrap();

    sqlx::query!("CREATE TABLE IF NOT EXISTS exclusions (guild_id INTEGER, channel_id INTEGER, PRIMARY KEY (guild_id, channel_id));")
        .execute(DB.get().unwrap())
        .await
        .unwrap();

    sqlx::query!("CREATE TABLE IF NOT EXISTS opted_out (user_id INTEGER PRIMARY KEY);")
        .execute(DB.get().unwrap())
        .await
        .unwrap();

    sqlx::query!("CREATE INDEX IF NOT EXISTS hooks_by_guild ON hooks (guild_id);")
        .execute(DB.get().unwrap())
        .await
        .unwrap();

    sqlx::query!("CREATE INDEX IF NOT EXISTS hooks_by_user ON hooks (user_id);")
        .execute(DB.get().unwrap())
        .await
        .unwrap();

    let token = env::var(if DEBUG { "TOKEN_DEBUG" } else { "TOKEN" }).expect("missing TOKEN from environment variable");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    let framework = Framework::builder()
        .options(FrameworkOptions {
            commands: vec![commands::list(), commands::add(), commands::remove(), commands::exclude(), commands::include(), commands::opt()],
            owners: HashSet::from([UserId::new(342429466359758850)]),
            event_handler: move |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();
    let client = ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}

async fn event_handler(ctx: &poise::serenity_prelude::Context, _event: &FullEvent, _framework: poise::FrameworkContext<'_, Data, Error>, _data: &Data) -> Result<(), Error>
{
    match _event {
        FullEvent::Message { new_message } => {
            handle_message(ctx, new_message.clone(), false).await;
        }
        FullEvent::MessageUpdate { event, .. } => {
            let mut message = Message::default();
            event.apply_to_message(&mut message);
            handle_message(ctx, message, true).await;
        }
        _ => {}
    }
    Ok(())
}

async fn handle_message(ctx: &poise::serenity_prelude::Context, message: Message, edit: bool)
{
    if message.author == **ctx.cache.current_user() {
        return;
    }
    let guild = message.guild_id;
    if guild.is_none() {
        return;
    }

    let author_id = message.author.id.get() as i64;
    let guild_id = guild.unwrap().get() as i64;
    let channel_id = message.channel_id.get() as i64;

    let excluded_channels = sqlx::query!("SELECT COUNT(*) AS count FROM exclusions WHERE guild_id = ? AND channel_id = ?", guild_id, channel_id)
        .fetch_one(DB.get().unwrap())
        .await
        .unwrap();

    let excluded_users = sqlx::query!("SELECT COUNT(*) AS count FROM opted_out WHERE user_id = ?", author_id)
        .fetch_one(DB.get().unwrap())
        .await
        .unwrap();

    if excluded_channels.count == 0 && excluded_users.count == 0 {
        let checks = sqlx::query!("SELECT hook, user_id FROM hooks WHERE guild_id = ? AND user_id <> ?", guild_id, author_id)
            .fetch_all(DB.get().unwrap())
            .await
            .unwrap();

        let content = message.content.clone();
        for record in checks {
            let str = record.hook.unwrap();
            if content.contains(&str) {
                let user_id = UserId::from(record.user_id.unwrap() as u64);
                let user = user_id.to_user(&ctx.http).await.unwrap();
                let guild_channel = message
                    .channel(ctx).await.unwrap()
                    .guild().unwrap();
                let guild = guild_channel.guild(ctx).unwrap().clone();
                let _member = guild.member(&ctx.http, user_id).await;
                let member;
                match _member {
                    Ok(m) => member = m,
                    Err(_) => continue
                }
                let user_perms = guild.user_permissions_in(&guild_channel, &*member.clone());
                if user_perms.read_message_history() & user_perms.view_channel() {
                    let message = CreateMessage::new().content(format!("Hook `{}` triggered in {}{}by {}", str, message.link(), if !edit { " " } else { " (edited) " }, message.author.mention()));
                    user.direct_message(ctx, message).await.expect(format!("failed to dm user {}", user_id).as_str());
                }
            }
        }
    }
}