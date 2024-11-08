use std::fmt::Debug;
use poise::CreateReply;
use poise::serenity_prelude::{CreateActionRow, CreateEmbed, CreateInteractionResponseMessage, CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption};
use crate::{Context, Error, DB};
use base64::{Engine as _, engine::{general_purpose}};

#[poise::command(slash_command, prefix_command, category = "Hooks")]
pub async fn list(
    ctx: Context<'_>,
) -> Result<(), Error>
{
    let user = ctx.author();
    let user_id = user.id.get() as i64;
    let builder = create_list_msg_reply(user_id).await;
    ctx.send(builder).await?;
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


struct ListMsg {
    embed: CreateEmbed,
    hook_list: Vec<CreateSelectMenuOption>,
}
pub async fn _create_list_msg(id: i64) -> ListMsg {
    let response_hooks = sqlx::query!("SELECT hook, guild_id FROM hooks WHERE user_id = ? ORDER BY hook DESC;", id)
        .fetch_all(DB.get().unwrap())
        .await
        .unwrap();
    let mut embed = CreateEmbed::new();
    let mut hook_list = vec![];
    for (i, hook) in response_hooks.iter().enumerate() {
        let real_hook = hook.hook.clone().unwrap();
        let guild_id = hook.guild_id.unwrap();
        let hook_title = format!("{}: {}", i, guild_id);
        let hook_remove_title = format!("Remove hook at index {}", i);
        embed = embed.field(hook_title, real_hook.clone(), false);
        hook_list.push(CreateSelectMenuOption::new(hook_remove_title, format!("{}|{}", guild_id, general_purpose::STANDARD.encode(real_hook))));
    }
    ListMsg {
        embed,
        hook_list,
    }
}

pub async fn create_list_msg_reply(id: i64) -> CreateReply {
    let list_msg = _create_list_msg(id).await;
    let hook_list = list_msg.hook_list.clone();
    let mut embed = list_msg.embed.clone();
    let builder;
    if !hook_list.is_empty() {
        let remove_dropdown = CreateSelectMenu::new("remove_hook_list", CreateSelectMenuKind::String {
            options: hook_list,
        }).placeholder("Remove hook...");
        builder = CreateReply::default()
            .embed(embed)
            .components(vec![CreateActionRow::SelectMenu(remove_dropdown)]);
    } else {
        embed = embed.description("You have no hooks!");
        builder = CreateReply::default()
            .embed(embed);
    }

    builder
}

pub async fn create_list_msg_interaction(id: i64) -> CreateInteractionResponseMessage {
    let list_msg = _create_list_msg(id).await;
    let hook_list = list_msg.hook_list.clone();
    let mut embed = list_msg.embed.clone();
    let builder;
    if !hook_list.is_empty() {
        let remove_dropdown = CreateSelectMenu::new("remove_hook_list", CreateSelectMenuKind::String {
            options: hook_list,
        }).placeholder("Remove hook...");
        builder = CreateInteractionResponseMessage::default()
            .embed(embed)
            .components(vec![CreateActionRow::SelectMenu(remove_dropdown)]);
    } else {
        embed = embed.description("You have no hooks!");
        builder = CreateInteractionResponseMessage::default()
            .embed(embed);
    }

    builder
}