use std::fmt;

/// Telonex download channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    Trades,
    Quotes,
    BookSnapshot5,
    BookSnapshot25,
    BookSnapshotFull,
    OnchainFills,
}

impl Channel {
    pub fn as_str(self) -> &'static str {
        match self {
            Channel::Trades => "trades",
            Channel::Quotes => "quotes",
            Channel::BookSnapshot5 => "book_snapshot_5",
            Channel::BookSnapshot25 => "book_snapshot_25",
            Channel::BookSnapshotFull => "book_snapshot_full",
            Channel::OnchainFills => "onchain_fills",
        }
    }
}

impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Channel {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "trades" => Ok(Channel::Trades),
            "quotes" => Ok(Channel::Quotes),
            "book_snapshot_5" => Ok(Channel::BookSnapshot5),
            "book_snapshot_25" => Ok(Channel::BookSnapshot25),
            "book_snapshot_full" => Ok(Channel::BookSnapshotFull),
            "onchain_fills" => Ok(Channel::OnchainFills),
            _ => Err(format!("unknown channel: {s}")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TelonexFile {
    pub exchange: String,
    pub channel: Channel,
    pub date: String,
    pub asset_id: String,
}
