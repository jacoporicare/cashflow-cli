use cashflow::Result;
use cashflow::cli::{BalanceAction, Cli, Commands, ConfigAction, OneTimeAction, RecurringAction};
use cashflow::commands::{
    execute_balance_set, execute_balance_show, execute_config_set_data_dir, execute_config_show,
    execute_export, execute_one_time_add, execute_one_time_delete, execute_one_time_edit,
    execute_one_time_list, execute_plan, execute_recurring_add, execute_recurring_delete,
    execute_recurring_disable, execute_recurring_edit, execute_recurring_enable,
    execute_recurring_list,
};
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        None => {
            execute_plan(30, false).await?;
        }
        Some(Commands::Plan { days, past }) => {
            execute_plan(*days, *past).await?;
        }

        Some(Commands::Balance { action }) => match action {
            BalanceAction::Set { amount, date } => {
                execute_balance_set(amount, date.as_deref()).await?;
            }
            BalanceAction::Show => {
                execute_balance_show().await?;
            }
        },

        Some(Commands::Recurring { action }) => match action {
            RecurringAction::Add {
                description,
                amount,
                day,
            } => {
                execute_recurring_add(description, amount, *day).await?;
            }
            RecurringAction::List => {
                execute_recurring_list().await?;
            }
            RecurringAction::Edit {
                id,
                amount,
                day,
                description,
            } => {
                execute_recurring_edit(id, amount.as_deref(), *day, description.as_deref()).await?;
            }
            RecurringAction::Disable { id } => {
                execute_recurring_disable(id).await?;
            }
            RecurringAction::Enable { id } => {
                execute_recurring_enable(id).await?;
            }
            RecurringAction::Delete { id } => {
                execute_recurring_delete(id).await?;
            }
        },

        Some(Commands::OneTime { action }) => match action {
            OneTimeAction::Add {
                description,
                amount,
                date,
            } => {
                execute_one_time_add(description, amount, date).await?;
            }
            OneTimeAction::List { upcoming } => {
                execute_one_time_list(*upcoming).await?;
            }
            OneTimeAction::Edit {
                id,
                amount,
                date,
                description,
            } => {
                execute_one_time_edit(
                    id,
                    amount.as_deref(),
                    date.as_deref(),
                    description.as_deref(),
                )
                .await?;
            }
            OneTimeAction::Delete { id } => {
                execute_one_time_delete(id).await?;
            }
        },

        Some(Commands::Export { format }) => {
            execute_export(format).await?;
        }

        Some(Commands::Config { action }) => match action {
            ConfigAction::Show => {
                execute_config_show().await?;
            }
            ConfigAction::SetDataDir { path } => {
                execute_config_set_data_dir(path).await?;
            }
        },
    }

    Ok(())
}
