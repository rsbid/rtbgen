use core::fmt;

use super::MAX_DEALS;
use super::MAX_NODES;
use crate::code::{DealAuctionType, FinalDecision, GppSectionId};
use crate::currency::Currency;
use crate::flag::Flag;
use crate::ids::DealId;
use crate::money::Money;
use crate::raw::RawJson;
use crate::version::SpecVersion;
use crate::write::{
    Object, field_id, field_number, field_string, object, object_field, option_currency,
    option_money, option_number, option_number_array, option_raw, option_string,
    option_string_array,
};

/// A private marketplace attached to an impression.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Pmp {
    private_auction: Option<Flag>,
    deals: DealList,
    ext: Option<RawJson>,
}

impl Pmp {
    /// Builds a marketplace with a fixed auction flag and its deals.
    pub(crate) const fn new(private_auction: Flag, deals: &DealList) -> Self {
        Self { private_auction: Some(private_auction), deals: *deals, ext: None }
    }

    pub(crate) fn write<W: fmt::Write>(&self, f: &mut Object<'_, W>) -> fmt::Result {
        option_number(f, "private_auction", self.private_auction)?;
        self.deals.write(f, "deals")?;
        option_raw(f, "ext", self.ext)
    }

    pub(crate) fn maximal() -> Self {
        let mut deals = DealList::default();
        deals.push(&Deal::maximal());

        Self {
            private_auction: Some(Flag::Yes),
            deals,
            ext: Some(RawJson::from_static(r#"{"k":1}"#)),
        }
    }
}

/// A private marketplace deal.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Deal {
    id: DealId,
    bidfloor: Option<Money>,
    bidfloorcur: Option<Currency>,
    at: Option<DealAuctionType>,
    wseat: &'static [&'static str],
    wadomain: &'static [&'static str],
    ext: Option<RawJson>,
}

impl Deal {
    /// Builds a deal with only its id set.
    pub(crate) const fn new(id: DealId) -> Self {
        Self {
            id,
            bidfloor: None,
            bidfloorcur: None,
            at: None,
            wseat: &[],
            wadomain: &[],
            ext: None,
        }
    }

    fn write<W: fmt::Write>(&self, f: &mut Object<'_, W>) -> fmt::Result {
        field_id(f, "id", self.id)?;
        option_money(f, "bidfloor", self.bidfloor)?;
        option_currency(f, "bidfloorcur", self.bidfloorcur)?;
        option_number(f, "at", self.at)?;
        option_string_array(f, "wseat", self.wseat)?;
        option_string_array(f, "wadomain", self.wadomain)?;
        option_raw(f, "ext", self.ext)
    }

    pub(crate) const fn maximal() -> Self {
        Self {
            id: DealId::new(1),
            bidfloor: Some(Money::from_scaled(15_000)),
            bidfloorcur: Some(Currency::USD),
            at: Some(DealAuctionType::AGREED_DEAL_PRICE),
            wseat: &["seat-1"],
            wadomain: &["adv.example"],
            ext: Some(RawJson::from_static(r#"{"k":1}"#)),
        }
    }
}

setters!(Deal {
    set_bidfloor => bidfloor: Option<Money>,
    set_bidfloorcur => bidfloorcur: Option<Currency>,
    set_at => at: Option<DealAuctionType>,
    set_wadomain => wadomain: &'static [&'static str],
});

/// A fixed-capacity list of deals.
#[derive(Clone, Copy, Debug)]
pub(crate) struct DealList {
    items: [Deal; MAX_DEALS],
    len: u8,
}

impl Default for DealList {
    fn default() -> Self {
        // The seed value is never written: `len` starts at zero.
        Self { items: [Deal::maximal(); MAX_DEALS], len: 0 }
    }
}

impl DealList {
    #[cfg(test)]
    pub(crate) const fn count(&self) -> u8 {
        self.len
    }

    pub(crate) fn push(&mut self, deal: &Deal) {
        if let Some(slot) = self.items.get_mut(usize::from(self.len)) {
            *slot = *deal;
            self.len = self.len.saturating_add(1);
        }
    }

    fn write<W: fmt::Write>(&self, f: &mut Object<'_, W>, name: &str) -> fmt::Result {
        if self.len == 0 {
            return Ok(());
        }

        f.key(name)?;
        f.out().write_char('[')?;

        let mut first = true;
        for deal in self.items.iter().take(usize::from(self.len)) {
            if first {
                first = false;
            } else {
                f.out().write_char(',')?;
            }
            object(f.out(), |f| deal.write(f))?;
        }

        f.out().write_char(']')
    }
}

/// The upstream source of the bid request.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Source {
    fd: Option<FinalDecision>,
    tid: Option<&'static str>,
    pchain: Option<&'static str>,
    schain: Option<SupplyChain>,
    ext: Option<RawJson>,
}

impl Source {
    pub(crate) fn write<W: fmt::Write>(
        &self,
        f: &mut Object<'_, W>,
        spec: SpecVersion,
    ) -> fmt::Result {
        option_number(f, "fd", self.fd)?;
        option_string(f, "tid", self.tid)?;
        option_string(f, "pchain", self.pchain)?;

        if spec.is_v26()
            && let Some(schain) = &self.schain
        {
            object_field(f, "schain", |f| schain.write(f))?;
        }

        option_raw(f, "ext", self.ext)
    }

    pub(crate) fn maximal() -> Self {
        let mut nodes = NodeList::default();
        nodes.push(&SupplyChainNode::maximal());

        Self {
            fd: Some(FinalDecision::UPSTREAM),
            tid: Some("tid-1"),
            pchain: Some("XYZ01234:ABCD56789"),
            schain: Some(SupplyChain {
                ver: "1.0",
                complete: Flag::Yes,
                nodes,
                ext: Some(RawJson::from_static(r#"{"k":1}"#)),
            }),
            ext: Some(RawJson::from_static(r#"{"k":1}"#)),
        }
    }
}

/// The supply chain object.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SupplyChain {
    ver: &'static str,
    complete: Flag,
    nodes: NodeList,
    ext: Option<RawJson>,
}

impl SupplyChain {
    fn write<W: fmt::Write>(&self, f: &mut Object<'_, W>) -> fmt::Result {
        field_string(f, "ver", self.ver)?;
        field_number(f, "complete", self.complete)?;
        self.nodes.write(f, "nodes")?;
        option_raw(f, "ext", self.ext)
    }
}

/// One node in the supply chain.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SupplyChainNode {
    asi: &'static str,
    sid: &'static str,
    rid: Option<&'static str>,
    hp: Option<Flag>,
    ext: Option<RawJson>,
}

impl SupplyChainNode {
    fn write<W: fmt::Write>(&self, f: &mut Object<'_, W>) -> fmt::Result {
        field_string(f, "asi", self.asi)?;
        field_string(f, "sid", self.sid)?;
        option_string(f, "rid", self.rid)?;
        option_number(f, "hp", self.hp)?;
        option_raw(f, "ext", self.ext)
    }

    pub(crate) const fn maximal() -> Self {
        Self {
            asi: "exchange.example",
            sid: "123",
            rid: Some("rid-1"),
            hp: Some(Flag::Yes),
            ext: Some(RawJson::from_static(r#"{"k":1}"#)),
        }
    }
}

/// A fixed-capacity list of supply-chain nodes.
#[derive(Clone, Copy, Debug)]
pub(crate) struct NodeList {
    items: [SupplyChainNode; MAX_NODES],
    len: u8,
}

impl Default for NodeList {
    fn default() -> Self {
        Self { items: [SupplyChainNode::maximal(); MAX_NODES], len: 0 }
    }
}

impl NodeList {
    pub(crate) fn push(&mut self, node: &SupplyChainNode) {
        if let Some(slot) = self.items.get_mut(usize::from(self.len)) {
            *slot = *node;
            self.len = self.len.saturating_add(1);
        }
    }

    fn write<W: fmt::Write>(&self, f: &mut Object<'_, W>, name: &str) -> fmt::Result {
        if self.len == 0 {
            return Ok(());
        }

        f.key(name)?;
        f.out().write_char('[')?;

        let mut first = true;
        for node in self.items.iter().take(usize::from(self.len)) {
            if first {
                first = false;
            } else {
                f.out().write_char(',')?;
            }
            object(f.out(), |f| node.write(f))?;
        }

        f.out().write_char(']')
    }
}

/// Privacy regulations that apply to the request.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Regs {
    coppa: Option<Flag>,
    gdpr: Option<Flag>,
    gpp: Option<&'static str>,
    gpp_sid: &'static [GppSectionId],
    ext: Option<RawJson>,
}

setters!(Regs {
    set_coppa => coppa: Option<Flag>,
    set_gdpr => gdpr: Option<Flag>,
    set_gpp => gpp: Option<&'static str>,
    set_gpp_sid => gpp_sid: &'static [GppSectionId],
    set_ext => ext: Option<RawJson>,
});

impl Regs {
    pub(crate) fn write<W: fmt::Write>(
        &self,
        f: &mut Object<'_, W>,
        spec: SpecVersion,
    ) -> fmt::Result {
        option_number(f, "coppa", self.coppa)?;

        if spec.is_v26() {
            option_number(f, "gdpr", self.gdpr)?;
            option_string(f, "gpp", self.gpp)?;
            option_number_array(f, "gpp_sid", self.gpp_sid)?;
        }

        option_raw(f, "ext", self.ext)
    }

    pub(crate) const fn maximal() -> Self {
        Self {
            coppa: Some(Flag::No),
            gdpr: Some(Flag::Yes),
            gpp: Some(crate::privacy::GPP_GRANTED),
            gpp_sid: &[GppSectionId::TCF_EU_V2],
            ext: Some(RawJson::from_static(r#"{"k":1}"#)),
        }
    }
}
