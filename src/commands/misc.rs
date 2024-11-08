use poise::CreateReply;
use crate::{Context, Error};
use poise::serenity_prelude::builder::{CreateEmbed};

#[poise::command(slash_command, prefix_command, category = "Misc")]
pub async fn help(
    ctx: Context<'_>,
) -> Result<(), Error>
{
    let embed = CreateEmbed::new()
        .title("Commands")
        .field("List", "List all of your hooks, with the option to remove them", true)
        .field("Add", "Add a new hook. Must be run in a server.", true)
        .field("Remove", "Remove a hook by index (indices from list).", true)
        .field("Include/Exclude", "Include or exclude a channel from being eligible to trigger hooks.", true)
        .field("Opt In/Out", "Opt in or out of having your messages eligible to trigger hooks.", true);
    let builder = CreateReply::default().embed(embed);
    ctx.send(builder).await?;
    Ok(())
}