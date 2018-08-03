//! Everything related to the uavcan ArrayInfo tag `ArrayInfo`

use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

/// Uavcan array information
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArrayInfo {
    /// Dynamic array on the less than form (i.e. `uint2[<5]`)
    DynamicLess(u64),
    /// Dynamic array on the less or equal form (i.e. `uint2[<=5]`)
    DynamicLeq(u64),
    /// Static array on the less or equal form (i.e. `uint2[5]`)
    Static(u64),
}

impl ArrayInfo {
    pub(crate) fn normalize(self) -> Self {
        // 4. For dynamic arrays, replace the max length specifier in the form [<X] to the form [<=Y]
        match self {
            ArrayInfo::DynamicLess(num) => ArrayInfo::DynamicLeq(num-1),
            x => x,
        }
    }
}

impl Display for ArrayInfo {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match *self {
            ArrayInfo::DynamicLess(ref num) => write!(f, "[<{}]", num),
            ArrayInfo::DynamicLeq(ref num) => write!(f, "[<={}]", num),
            ArrayInfo::Static(ref num) => write!(f, "[{}]", num),
        }
    }
}