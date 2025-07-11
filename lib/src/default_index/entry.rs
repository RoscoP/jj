// Copyright 2023 The Jujutsu Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(missing_docs)]

use std::fmt::Debug;
use std::fmt::Formatter;
use std::hash::Hash;
use std::hash::Hasher;

use smallvec::SmallVec;

use super::composite::CompositeIndex;
use super::composite::DynIndexSegment;
use crate::backend::ChangeId;
use crate::backend::CommitId;
use crate::object_id::ObjectId as _;

/// Global index position.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub(super) struct IndexPosition(pub(super) u32);

impl IndexPosition {
    pub const MIN: Self = IndexPosition(u32::MIN);
    pub const MAX: Self = IndexPosition(u32::MAX);
}

/// Local position within an index segment.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub(super) struct LocalPosition(pub(super) u32);

// SmallVec reuses two pointer-size fields as inline area, which meas we can
// inline up to 16 bytes (on 64-bit platform) for free.
pub(super) type SmallIndexPositionsVec = SmallVec<[IndexPosition; 4]>;
pub(super) type SmallLocalPositionsVec = SmallVec<[LocalPosition; 4]>;

#[derive(Clone)]
pub(super) struct IndexEntry<'a> {
    source: &'a DynIndexSegment,
    pos: IndexPosition,
    /// Position within the source segment
    local_pos: LocalPosition,
}

impl Debug for IndexEntry<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IndexEntry")
            .field("pos", &self.pos)
            .field("local_pos", &self.local_pos)
            .field("commit_id", &self.commit_id().hex())
            .finish()
    }
}

impl PartialEq for IndexEntry<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos
    }
}

impl Eq for IndexEntry<'_> {}

impl Hash for IndexEntry<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pos.hash(state);
    }
}

impl<'a> IndexEntry<'a> {
    pub(super) fn new(
        source: &'a DynIndexSegment,
        pos: IndexPosition,
        local_pos: LocalPosition,
    ) -> Self {
        IndexEntry {
            source,
            pos,
            local_pos,
        }
    }

    pub fn position(&self) -> IndexPosition {
        self.pos
    }

    pub fn generation_number(&self) -> u32 {
        self.source.generation_number(self.local_pos)
    }

    pub fn commit_id(&self) -> CommitId {
        self.source.commit_id(self.local_pos)
    }

    pub fn change_id(&self) -> ChangeId {
        self.source.change_id(self.local_pos)
    }

    pub fn num_parents(&self) -> u32 {
        self.source.num_parents(self.local_pos)
    }

    pub fn parent_positions(&self) -> SmallIndexPositionsVec {
        self.source.parent_positions(self.local_pos)
    }

    pub fn parents(&self) -> impl ExactSizeIterator<Item = IndexEntry<'a>> + use<'a> {
        let composite = CompositeIndex::new(self.source);
        self.parent_positions()
            .into_iter()
            .map(move |pos| composite.entry_by_pos(pos))
    }
}
