//! Pure, testable options for an opt-in, automatically cleaned desktop-host probe.
//! There is no mode that requests a new WorkerW or edits Explorer styles.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Inspect,
    Attach { duration_ms: u64 },
    AttachModern { duration_ms: u64 },
}

/// Explicit platform strategies; no caller may silently fall back from one
/// cross-process desktop host to the other.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttachTarget {
    WorkerW,
    ProgmanLayered,
}

pub fn select_target(
    mode: Mode,
    raised_desktop: bool,
) -> Result<Option<AttachTarget>, &'static str> {
    match mode {
        Mode::Inspect => Ok(None),
        Mode::Attach { .. } if !raised_desktop => Ok(Some(AttachTarget::WorkerW)),
        Mode::AttachModern { .. } if raised_desktop => Ok(Some(AttachTarget::ProgmanLayered)),
        Mode::Attach { .. } => {
            Err("legacy WorkerW probe is unsupported on a raised desktop; use --attach-modern")
        }
        Mode::AttachModern { .. } => Err("layered Progman probe is limited to raised desktops"),
    }
}

/// A bounded opt-in. Never sends undocumented Progman messages or changes
/// Explorer window styles or process state.
pub fn parse_args<I, S>(args: I) -> Result<Mode, &'static str>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut args = args.into_iter();
    let first = args.next();
    match first.as_ref().map(AsRef::as_ref) {
        None | Some("--inspect") => {
            if args.next().is_some() {
                return Err("--inspect cannot be combined with other options");
            }
            Ok(Mode::Inspect)
        }
        Some("--attach") | Some("--attach-modern") => {
            let modern = first
                .as_ref()
                .is_some_and(|mode| mode.as_ref() == "--attach-modern");
            let duration_ms = match args.next() {
                None => 3000,
                Some(flag) if flag.as_ref() == "--duration-ms" => {
                    let value = args.next().ok_or("missing --duration-ms value")?;
                    if args.next().is_some() {
                        return Err("unexpected extra argument");
                    }
                    value
                        .as_ref()
                        .parse::<u64>()
                        .map_err(|_| "invalid duration")?
                }
                Some(_) => return Err("unsupported attach argument"),
            };
            if !(1..=10_000).contains(&duration_ms) {
                return Err("duration must be 1..=10000 ms");
            }
            if modern {
                Ok(Mode::AttachModern { duration_ms })
            } else {
                Ok(Mode::Attach { duration_ms })
            }
        }
        Some(_) => Err("unknown option"),
    }
}
