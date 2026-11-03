//! Pure, testable options for an opt-in, automatically cleaned desktop-host probe.
//! There is no mode that requests a new WorkerW or edits Explorer styles.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Inspect,
    Attach { duration_ms: u64 },
}

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
        Some("--attach") => match args.next() {
            None => Ok(Mode::Attach { duration_ms: 3000 }),
            Some(flag) if flag.as_ref() == "--duration-ms" => {
                let value = args.next().ok_or("missing --duration-ms value")?;
                if args.next().is_some() {
                    return Err("unexpected extra argument");
                }
                let value = value
                    .as_ref()
                    .parse::<u64>()
                    .map_err(|_| "invalid duration")?;
                if !(1..=10_000).contains(&value) {
                    return Err("duration must be 1..=10000 ms");
                }
                Ok(Mode::Attach { duration_ms: value })
            }
            Some(_) => Err("unsupported attach argument"),
        },
        Some(_) => Err("unknown option"),
    }
}
