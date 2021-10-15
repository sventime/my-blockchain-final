#[derive(Debug, Default, Clone)]
pub struct Node<T> {
    data: T,
    prev: Option<Box<Node<T>>>,
}

impl<T> Node<T> {
    pub fn new(data: T, prev: Option<Box<Node<T>>>) -> Self {
        Self { data, prev }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Chain<T: Default> {
    head: Option<Box<Node<T>>>,
    len: usize,
}

impl<T: Default> Chain<T> {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn append(&mut self, item: T) {
        let head = self.head.take();
        let node = Box::new(Node::new(item, head));

        self.head = Some(node);
        self.len += 1;
    }

    pub fn head(&self) -> Option<&T> {
        match &self.head {
            None => None,
            Some(head) => Some(&head.data),
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn iter(&self) -> ChainIter<T> {
        ChainIter {
            next: self.head.as_deref(),
        }
    }

    pub fn iter_mut(&mut self) -> ChainIterMut<T> {
        ChainIterMut {
            next: self.head.as_deref_mut(),
        }
    }
}

pub struct ChainIter<'a, T> {
    next: Option<&'a Node<T>>,
}
pub struct ChainIterMut<'a, T> {
    next: Option<&'a mut Node<T>>,
}

impl<'a, T> Iterator for ChainIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node| {
            self.next = node.prev.as_deref();
            &node.data
        })
    }
}

impl<'a, T> Iterator for ChainIterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node| {
            self.next = node.prev.as_deref_mut();
            &mut node.data
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain() {
        let mut chain = Chain::<u32>::new();
        chain.append(1);
        chain.append(2);
        chain.append(10);
        let mut iter = chain.iter();

        assert_eq!(iter.next(), Some(&10));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(chain.head(), Some(&10));
        assert_eq!(chain.len(), 3);
    }
}
