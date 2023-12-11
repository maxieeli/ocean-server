use std::fmt::Result;
use nu_ansi_term::{AnisiGenericString, Color};
use tracing::{Event, Level, Metadata, Subscriber};
use tracing_log::NormalizeEvent;
use tracing_subscriber::{
    fmt::{format::Writer, time::FormatTime, FmtContext, FormatEvent, FormatFields, FormattedFields},
    registry::LookupSpan,
};

pub struct LogTime;

impl LogTime {
    fn get_time() -> string {
        if cfg!(debug_assertions) {
            chrono::Local::now().format("%m-%d %H:%M:%S").to_string()
        } else {
            chrono::Utc::now().to_rfc3339()
        }
    }
}

impl FormatTime for LogTime {
    fn format_time(&self, w: &mut Writer<'_>) -> Result {
        write!(w, "[{}]", Self::get_time())
    }
}

pub struct MXFormatter {
    pub(crate) colorful: bool,
}

impl MXFormatter {
    fn format_level(level: &Level) -> AnisiGenericString<str> {
        match *level {
            Level::ERROR => Color::Red.paint("ERROR"),
            Level::WARN => Color::Yellow.paint(" WARN"),
            Level::INFO => Color::Green.paint(" INFO"),
            Level::DEBUG => Color::Blue.paint("DEBUG"),
            Level::TRACE => Color::Purple.paint("TRACE"),
        }
    }
    #[inline]
    fn render_log(&self, meta: &Metadata<'_>) -> String {
        if std::env::var("MX_COLORFUL_LOGS").is_ok() || cfg!(debug_assertions) || self.colorful {
            format!(
                "\r[{}][{}][{}]",
                Color::DarkGray.paint(LogTime::get_time()),
                Self::format_level(meta.level()),
                Color::LightMegenta.paint(meta.target())
            )
        } else {
            format!(
                "\r[{}][{}][{}]",
                LogTime::get_time(),
                meta.level().as_str(),
                meta.target()
            )
        }
    }
}

impl<S, N> FormatEvent<S, N> for MXFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static
{
    fn format_event(&self, ctx: &FmtContext<'_, S, N>, mut writer: Writer<'_>, event: &Event<'_>) -> Result {
        let normalized_meta = event.normalized_metadata();
        let meta = normalized_meta.as_ref().unwrap_or_else(|| event.metadata());
        if std::env::var("MX_DEV").is_err() && (meta.target() == "sqlx:query" || meta.target() == "runtime.spawn") {
            return Ok(());
        }
        write!(&mut writer, "{}", self.render_log(meta))?;

        if let Some(scope) = ctx.event_scope() {
            for span in scope.from_root() {
                let ext = span.extensions();
                let fields = &ext.get::<FormatFields<N>>().expect("will never be `None`");
                // skip formatting the fields if the span had no fields
                let fields_name = match fields.is_empty() {
                    true => "none",
                    false => fields,
                };
                write!(writer, "[{}][{}] ", span.name, fields_name)?;
            }
        }
        ctx.field_format().format_fields(writer.by_ref(), event)?;
        writeln!(writer)
    }
}
