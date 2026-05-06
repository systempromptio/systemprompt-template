use systemprompt_template as _;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Box::pin(systemprompt_template::cli::run()).await
}
