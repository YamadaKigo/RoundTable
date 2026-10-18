use crate::ecs::Entity;
use std::ops::{Deref, DerefMut, Index, IndexMut};

pub(crate) struct ComponentTable<T> {
    entities: Vec<Entity>,
    values: Vec<T>,
}

impl<T> Default for ComponentTable<T> {
    fn default() -> Self {
        Self {
            entities: Vec::new(),
            values: Vec::new(),
        }
    }
}

impl<T> ComponentTable<T> {
    pub(crate) fn push(&mut self, entity: Entity, value: T) {
        self.entities.push(entity);
        self.values.push(value);
    }

    pub(crate) fn replace(&mut self, index: usize, entity: Entity, value: T) {
        self.entities[index] = entity;
        self.values[index] = value;
    }

    #[cfg(test)]
    pub(crate) fn entity(&self, index: usize) -> Option<Entity> {
        self.entities.get(index).copied()
    }
}

impl<T> Deref for ComponentTable<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.values
    }
}

impl<T> DerefMut for ComponentTable<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.values
    }
}

impl<T> Index<usize> for ComponentTable<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.values[index]
    }
}

impl<T> IndexMut<usize> for ComponentTable<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.values[index]
    }
}

impl<'a, T> IntoIterator for &'a ComponentTable<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut ComponentTable<T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.iter_mut()
    }
}
