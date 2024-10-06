use std::fmt::Write;
use poise::serenity_prelude::Channel;
use crate::{Context, Error, DB};

#[poise::command(slash_command, prefix_command, category = "Hooks")]
pub async fn list(
    ctx: Context<'_>,
) -> Result<(), Error>
{
    let user = ctx.author();
    let user_id = user.id.get() as i64;
    let response = sqlx::query!("SELECT hook, guild_id FROM hooks WHERE user_id = ? ORDER BY hook DESC;", user_id).fetch_all(DB.get().unwrap()).await.unwrap();
    let mut message = format!("Hooks:{}", "\n");
    for (i, hook) in response.iter().enumerate() {
        let real_hook = hook.hook.clone().unwrap();
        let hook_id = hook.guild_id.unwrap();
        writeln!(message, "Index: {}. Server: {}. Hook: `{}`", i, hook_id, real_hook).unwrap();
    }
    ctx.reply(message).await.expect("could not send message");
    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only, category = "Hooks")]
pub async fn add(
    ctx: Context<'_>,
    #[description = "Message to hook on"] hook: String,
) -> Result<(), Error>
{
    let user = ctx.author();
    let user_id = user.id.get() as i64;
    let guild = ctx
        .partial_guild()
        .await
        .unwrap();
    let guild_id = guild.id.get() as i64;
    sqlx::query!("INSERT INTO hooks VALUES (?, ?, ?)", user_id, guild_id, hook).execute(DB.get().unwrap()).await.unwrap();
    ctx.reply("Added hook!").await.expect("could not send message");
    Ok(())
}

#[poise::command(prefix_command, slash_command, category = "Hooks")]
pub async fn remove(
    ctx: Context<'_>,
    #[description = "Index to remove. You can find indexes by using the 'list' command."] index: usize,
) -> Result<(), Error>
{
    let user = ctx.author();
    let user_id = user.id.get() as i64;
    let response = sqlx::query!("SELECT hook, guild_id FROM hooks WHERE user_id = ? ORDER BY hook DESC;", user_id).fetch_all(DB.get().unwrap()).await.unwrap();
    if index >= response.len() {
        ctx.reply("Invalid index provided.").await.expect("could not send message");
        return Ok(());
    }

    let hook = &response[index];
    let msg = hook.hook.clone().unwrap();

    let guild_id = ctx.guild_id().unwrap().get() as i64;

    sqlx::query!("DELETE FROM hooks WHERE user_id = ? AND guild_id = ? AND hook = ?;", user_id, guild_id, msg).execute(DB.get().unwrap()).await.unwrap();
    ctx.reply("Removed hook!").await.expect("could not send message");
    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only, category = "Exclusions")]
pub async fn exclude(
    ctx: Context<'_>,
    #[description = "Channel to exclude. Defaults to the current channel."] channel: Option<Channel>) -> Result<(), Error>
{
    let channel_id = if channel.is_none() { ctx.channel_id() } else { channel.unwrap().id() };
    let guild_id = ctx.guild_id().unwrap();
    if guild_id != channel_id.to_channel(ctx).await.unwrap().guild().unwrap().guild_id {
        ctx.reply("Please run this command in the guild that has the channel you wish to exclude.").await.expect("could not send message");
        return Ok(());
    }
    let _channel_id = channel_id.get() as i64;
    let _guild_id = guild_id.get() as i64;
    sqlx::query!("INSERT INTO exclusions VALUES (?, ?)", _guild_id, _channel_id)
        .execute(DB.get().unwrap())
        .await
        .unwrap();

    ctx.reply("Excluded channel.").await.expect("could not send message");
    Ok(())
}

#[poise::command(prefix_command, slash_command, category = "Exclusions")]
pub async fn include(
    ctx: Context<'_>,
    #[description = "Channel to include. Defaults to the current channel."] channel: Option<Channel>,
) -> Result<(), Error>
{
    let channel_id = if channel.is_none() { ctx.channel_id() } else { channel.unwrap().id() };
    let guild_id = ctx.guild_id().unwrap();
    if guild_id != channel_id.to_channel(ctx).await.unwrap().guild().unwrap().guild_id {
        ctx.reply("Please run this command in the guild that has the channel you wish to include.").await.expect("could not send message");
        return Ok(());
    }
    let _channel_id = channel_id.get() as i64;
    let _guild_id = guild_id.get() as i64;
    let res = sqlx::query!("SELECT COUNT(*) AS count FROM exclusions WHERE guild_id = ? AND channel_id = ?", _guild_id, _channel_id)
        .fetch_one(DB.get().unwrap())
        .await
        .unwrap();

    if res.count > 0 {
        sqlx::query!("DELETE FROM exclusions WHERE guild_id = ? AND channel_id = ?", _guild_id, _channel_id)
            .execute(DB.get().unwrap())
            .await
            .unwrap();
        ctx.say("Removed channel from exclusions.").await.expect("could not send message");
    } else {
        ctx.say("Provided channel hasn't been excluded.").await.expect("could not send message");
    }
    Ok(())
}

#[derive(Debug, poise::ChoiceParameter)]
enum OptChoice {
    #[name = "In"]
    In,
    #[name = "Out"]
    Out,
}

#[poise::command(prefix_command, slash_command, category = "Exclusions")]
pub async fn opt(
    ctx: Context<'_>,
    #[description = "Opt in or out of having your messages trigger hooks."] choice: OptChoice,
) -> Result<(), Error>
{
    let user_id = ctx.author().id.get() as i64;
    match choice {
        OptChoice::In => {
            let results = sqlx::query!("SELECT COUNT(*) AS count FROM opted_out WHERE user_id = ?", user_id)
                .fetch_one(DB.get().unwrap())
                .await
                .unwrap();
            if results.count > 0 {
                sqlx::query!("DELETE FROM opted_out WHERE user_id = ?", user_id).execute(DB.get().unwrap())
                    .await
                    .unwrap();
                ctx.say("Opted you back in.").await.expect("could not send message");
            } else {
                ctx.say("You are not opted out.").await.expect("could not send message");
            }
        }
        OptChoice::Out => {
            let results = sqlx::query!("SELECT COUNT(*) AS count FROM opted_out WHERE user_id = ?", user_id)
                .fetch_one(DB.get().unwrap())
                .await
                .unwrap();
            if results.count == 0 {
                sqlx::query!("INSERT INTO opted_out VALUES (?)", user_id)
                    .execute(DB.get().unwrap())
                    .await
                    .unwrap();
                ctx.say("Successfully opted you out.").await.expect("could not send message");
            } else {
                ctx.say("You are already opted out.").await.expect("could not send message");
            }
        }
    }
    Ok(())
}