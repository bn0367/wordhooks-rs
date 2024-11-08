use crate::{Context, Error, DB};

#[derive(Debug, poise::ChoiceParameter)]
enum OptChoice {
    #[name = "In"]
    In,
    #[name = "Out"]
    Out,
}

#[poise::command(prefix_command, slash_command, category = "User")]
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