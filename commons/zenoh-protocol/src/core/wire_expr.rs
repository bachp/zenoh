//
// Copyright (c) 2022 ZettaScale Technology
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
//

//! This module defines the wire representation of Key Expressions.
use crate::core::ExprId;
use alloc::borrow::Cow;
use core::{convert::TryInto, fmt};
use zenoh_core::{bail, Result as ZResult};

/// A zenoh **resource** is represented by a pair composed by a **key** and a
/// **value**, such as, ```(/car/telemetry/speed, 320)```.  A **resource key**
/// is an arbitrary array of characters, with the exclusion of the symbols
/// ```*```, ```**```, ```?```, ```[```, ```]```, and ```#```,
/// which have special meaning in the context of zenoh.
///
/// A key including any number of the wildcard symbols, ```*``` and ```**```,
/// such as, ```/car/telemetry/*```, is called a **key expression** as it
/// denotes a set of keys. The wildcard character ```*``` expands to an
/// arbitrary string not including zenoh's reserved characters and the ```/```
/// character, while the ```**``` expands to  strings that may also include the
/// ```/``` character.  
///
/// Finally, it is worth mentioning that for time and space efficiency matters,
/// zenoh will automatically map key expressions to small integers. The mapping is automatic,
/// but it can be triggered excplicily by with `zenoh::Session::declare_keyexpr()`.
///
//
//  7 6 5 4 3 2 1 0
// +-+-+-+-+-+-+-+-+
// ~      id       — if Expr: id=0
// +-+-+-+-+-+-+-+-+
// ~    suffix     ~ if flag K==1 in Message's header
// +---------------+
//
#[derive(PartialEq, Eq, Hash, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct WireExpr<'a> {
    pub scope: ExprId, // 0 marks global scope
    pub suffix: Cow<'a, str>,
}

impl<'a> WireExpr<'a> {
    pub fn as_str(&'a self) -> &'a str {
        if self.scope == 0 {
            self.suffix.as_ref()
        } else {
            "<encoded_expr>"
        }
    }

    pub fn try_as_str(&'a self) -> ZResult<&'a str> {
        if self.scope == 0 {
            Ok(self.suffix.as_ref())
        } else {
            bail!("Scoped key expression")
        }
    }

    pub fn as_id(&'a self) -> ExprId {
        self.scope
    }

    pub fn try_as_id(&'a self) -> ZResult<ExprId> {
        if self.has_suffix() {
            bail!("Suffixed key expression")
        } else {
            Ok(self.scope)
        }
    }

    pub fn as_id_and_suffix(&'a self) -> (ExprId, &'a str) {
        (self.scope, self.suffix.as_ref())
    }

    pub fn has_suffix(&self) -> bool {
        !self.suffix.as_ref().is_empty()
    }

    pub fn to_owned(&self) -> WireExpr<'static> {
        WireExpr {
            scope: self.scope,
            suffix: self.suffix.to_string().into(),
        }
    }

    pub fn with_suffix(mut self, suffix: &'a str) -> Self {
        if self.suffix.is_empty() {
            self.suffix = suffix.into();
        } else {
            self.suffix += suffix;
        }
        self
    }
}

impl TryInto<String> for WireExpr<'_> {
    type Error = zenoh_core::Error;
    fn try_into(self) -> Result<String, Self::Error> {
        if self.scope == 0 {
            Ok(self.suffix.into_owned())
        } else {
            bail!("Scoped key expression")
        }
    }
}

impl TryInto<ExprId> for WireExpr<'_> {
    type Error = zenoh_core::Error;
    fn try_into(self) -> Result<ExprId, Self::Error> {
        self.try_as_id()
    }
}

impl fmt::Debug for WireExpr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.scope == 0 {
            write!(f, "{}", self.suffix)
        } else {
            write!(f, "{}:{}", self.scope, self.suffix)
        }
    }
}

impl fmt::Display for WireExpr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.scope == 0 {
            write!(f, "{}", self.suffix)
        } else {
            write!(f, "{}:{}", self.scope, self.suffix)
        }
    }
}

impl<'a> From<&WireExpr<'a>> for WireExpr<'a> {
    #[inline]
    fn from(key: &WireExpr<'a>) -> WireExpr<'a> {
        key.clone()
    }
}

impl From<ExprId> for WireExpr<'_> {
    #[inline]
    fn from(rid: ExprId) -> WireExpr<'static> {
        WireExpr {
            scope: rid,
            suffix: "".into(),
        }
    }
}

impl From<&ExprId> for WireExpr<'_> {
    #[inline]
    fn from(rid: &ExprId) -> WireExpr<'static> {
        WireExpr {
            scope: *rid,
            suffix: "".into(),
        }
    }
}

impl<'a> From<&'a str> for WireExpr<'a> {
    #[inline]
    fn from(name: &'a str) -> WireExpr<'a> {
        WireExpr {
            scope: 0,
            suffix: name.into(),
        }
    }
}

impl From<String> for WireExpr<'_> {
    #[inline]
    fn from(name: String) -> WireExpr<'static> {
        WireExpr {
            scope: 0,
            suffix: name.into(),
        }
    }
}

impl<'a> From<&'a String> for WireExpr<'a> {
    #[inline]
    fn from(name: &'a String) -> WireExpr<'a> {
        WireExpr {
            scope: 0,
            suffix: name.into(),
        }
    }
}

impl WireExpr<'_> {
    #[cfg(feature = "test")]
    pub fn rand() -> Self {
        use rand::{
            distributions::{Alphanumeric, DistString},
            Rng,
        };

        const MIN: usize = 2;
        const MAX: usize = 64;

        let mut rng = rand::thread_rng();

        let scope: ExprId = rng.gen_range(0..20);
        let suffix: String = if rng.gen_bool(0.5) {
            let len = rng.gen_range(MIN..MAX);
            Alphanumeric.sample_string(&mut rng, len)
        } else {
            String::new()
        };

        WireExpr {
            scope,
            suffix: suffix.into(),
        }
    }
}
