use std::cmp::min;
use std::collections::{hash_map::Entry, HashMap, HashSet};
use std::ops::Deref;
use std::sync::Arc;

use parking_lot::RwLock;
use rand::rngs::ThreadRng;
use rand::Rng;

use crate::node::{Peer, PeerId, PeerInner};
use crate::ErrorKind;

pub struct Membership {
    peers: Arc<RwLock<HashMap<PeerId, PeerInner>>>,
}

impl Membership {
    pub fn add<'p>(&self, peer: &'p Peer) -> crate::Result<&'p Peer> {
        let (id, inner) = peer.clone().into();

        match self.peers.write().entry(id) {
            Entry::Vacant(entry) => {
                entry.insert(inner);
                Ok(peer)
            }
            Entry::Occupied(_) => Err(ErrorKind::KnownMember((id, inner).into()).into()),
        }
    }

    pub fn random(&self, samples: usize, to_ignore: &HashSet<PeerId>) -> Option<Vec<Peer>> {
        let peers = self.peers.read();

        let peers: Vec<(&PeerId, &PeerInner)> = peers
            .iter()
            // skip myself
            // .filter(|(id, _)| *id != &self.sender.id())
            // skip infected peers
            .filter(|(id, _)| !to_ignore.iter().any(|peer| *id == peer))
            .map(|(id, peer)| (id, peer))
            .collect();

        let peers_len = peers.len();

        match peers_len {
            0 => None,
            1 => Some(vec![Peer::from(*peers.get(0).unwrap())]),
            _ => {
                let rand_idx = Membership::rand_indexes(samples, peers_len);
                Some(
                    rand_idx
                        .iter()
                        .map(|i| Peer::from(*peers.get(*i).unwrap()))
                        .collect(),
                )
            }
        }
    }

    fn rand_indexes(samples: usize, high: usize) -> HashSet<usize> {
        let samples = min(samples, high);
        let mut rand_idx: HashSet<usize> = HashSet::with_capacity(samples);
        loop {
            match rand_idx.len() {
                len if { len < samples } => {
                    rand_idx.insert(ThreadRng::default().gen_range(0, high));
                }
                _ => break,
            }
        }
        rand_idx
    }
}

#[cfg(test)]
mod tests {
    use crate::node::Address;

    use super::*;

    #[test]
    pub fn random_samples() {
        let mut peers = HashMap::with_capacity(10);
        for i in 0..10 {
            peers.insert(PeerId::default(), PeerInner::new(Address(i.to_string())));
        }

        let membership = Membership {
            peers: Arc::new(RwLock::new(peers)),
        };

        let samples = membership.random(4, &HashSet::new());
        assert!(samples.is_some());

        let samples = samples.unwrap();

        assert_eq!(4, samples.len());
    }
}