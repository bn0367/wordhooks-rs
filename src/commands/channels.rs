use poise::serenity_prelude::Channel;
use crate::{Context, Error, DB};

#[poise::command(slash_command, prefix_command, guild_only, category = "Channels")]
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

#[poise::command(prefix_command, slash_command, category = "Channels")]
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