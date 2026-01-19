use anyhow::Result;
use tracing_subscriber::fmt::format::FmtSpan;

pub fn init() -> Result<()> {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::NONE)
        .with_target(false)
        .init();
    Ok(())
}
