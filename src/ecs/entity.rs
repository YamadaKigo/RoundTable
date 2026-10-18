#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct Entity(u64);

#[derive(Default)]
pub(crate) struct EntityAllocator {
    next: u64,
}

impl EntityAllocator {
    pub(crate) fn spawn(&mut self) -> Entity {
        let entity = Entity(self.next);
        self.next += 1;
        entity
    }
}
