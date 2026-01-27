use systemprompt_template as _;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    systemprompt_template::cli::run().await
}
