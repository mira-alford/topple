pub trait Lattice: Clone + Eq {
    const BOTTOM: Self;
    const TOP: Self;
    fn join(self, other: Self) -> Self;
    fn meet(self, other: Self) -> Self;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlatLattice<T: Eq + Clone> {
    Top,
    Value(T),
    Bottom,
}

impl<T: Eq + Clone> Lattice for FlatLattice<T> {
    const BOTTOM: Self = FlatLattice::Bottom;
    const TOP: Self = FlatLattice::Top;

    fn join(self, other: Self) -> Self {
        match (&self, &other) {
            (FlatLattice::Value(a), FlatLattice::Value(b)) if a == b => self,
            (FlatLattice::Value(_), FlatLattice::Value(_)) => FlatLattice::Top,
            (FlatLattice::Top, _) | (_, FlatLattice::Top) => FlatLattice::Top,
            (FlatLattice::Bottom, _) => other,
            (_, FlatLattice::Bottom) => self,
        }
    }

    fn meet(self, other: Self) -> Self {
        match (&self, &other) {
            (FlatLattice::Value(a), FlatLattice::Value(b)) if a == b => self,
            (FlatLattice::Value(_), FlatLattice::Value(_)) => FlatLattice::Bottom,
            (FlatLattice::Bottom, _) | (_, FlatLattice::Bottom) => FlatLattice::Bottom,
            (FlatLattice::Top, _) => other,
            (_, FlatLattice::Top) => self,
        }
    }
}

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LatticeMap<K, L: Lattice> {
    BottomMap(BTreeMap<K, L>),
    TopMap(BTreeMap<K, L>),
}

impl<K: Ord + Clone, L: Lattice> LatticeMap<K, L> {
    pub fn read(&self, key: &K) -> L {
        match self {
            LatticeMap::BottomMap(m) => m.get(key).cloned().unwrap_or(L::BOTTOM),
            LatticeMap::TopMap(m) => m.get(key).cloned().unwrap_or(L::TOP),
        }
    }

    pub fn update(self, key: K, value: L) -> Self {
        match self {
            LatticeMap::BottomMap(mut m) => {
                m.insert(key, value);
                LatticeMap::BottomMap(m)
            }
            LatticeMap::TopMap(mut m) => {
                m.insert(key, value);
                LatticeMap::TopMap(m)
            }
        }
    }
}

impl<K: Ord + Clone, L: Lattice> Lattice for LatticeMap<K, L> {
    const BOTTOM: Self = LatticeMap::BottomMap(BTreeMap::new());
    const TOP: Self = LatticeMap::TopMap(BTreeMap::new());

    fn join(self, other: Self) -> Self {
        match (self, other) {
            (LatticeMap::BottomMap(mut a), LatticeMap::BottomMap(mut b)) => {
                if a.len() > b.len() {
                    (a, b) = (b, a);
                }
                for (k, v) in a.into_iter() {
                    if let Some(v2) = b.remove(&k) {
                        b.insert(k, L::join(v, v2));
                    } else {
                        b.insert(k, v);
                    }
                }
                LatticeMap::BottomMap(b)
            }

            (LatticeMap::BottomMap(a), LatticeMap::TopMap(mut b))
            | (LatticeMap::TopMap(mut b), LatticeMap::BottomMap(a)) => {
                for (k, v) in a.into_iter() {
                    if let Some(v2) = b.remove(&k) {
                        b.insert(k, L::join(v, v2));
                    }
                }
                LatticeMap::TopMap(b)
            }

            (LatticeMap::TopMap(mut a), LatticeMap::TopMap(mut b)) => {
                if a.len() > b.len() {
                    (a, b) = (b, a);
                }
                for (k, v) in a.into_iter() {
                    if let Some(v2) = b.remove(&k) {
                        b.insert(k, L::join(v, v2));
                    }
                }
                LatticeMap::TopMap(b)
            }
        }
    }

    fn meet(self, other: Self) -> Self {
        match (self, other) {
            (LatticeMap::BottomMap(mut a), LatticeMap::BottomMap(mut b)) => {
                if a.len() > b.len() {
                    (a, b) = (b, a);
                }
                for (k, v) in a.into_iter() {
                    if let Some(v2) = b.remove(&k) {
                        b.insert(k, L::meet(v, v2));
                    } else {
                        b.insert(k, v);
                    }
                }
                LatticeMap::BottomMap(b)
            }

            (LatticeMap::BottomMap(mut a), LatticeMap::TopMap(b))
            | (LatticeMap::TopMap(b), LatticeMap::BottomMap(mut a)) => {
                for (k, v) in b.into_iter() {
                    if let Some(v2) = a.remove(&k) {
                        a.insert(k, L::join(v, v2));
                    }
                }
                LatticeMap::TopMap(a)
            }

            (LatticeMap::TopMap(mut a), LatticeMap::TopMap(mut b)) => {
                if a.len() > b.len() {
                    (a, b) = (b, a);
                }
                for (k, v) in a.into_iter() {
                    if let Some(v2) = b.remove(&k) {
                        b.insert(k, L::meet(v, v2));
                    } else {
                        b.insert(k, v);
                    }
                }
                LatticeMap::TopMap(b)
            }
        }
    }
}
