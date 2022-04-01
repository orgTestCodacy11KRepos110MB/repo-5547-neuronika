use super::{reduce, Backward, Broadcast, Forward, Gradient, Shared};
use ndarray::{Array, DimMax, Dimension, Zip};
use std::rc::Rc;

pub(crate) struct Addition<D, E>
where
    D: Dimension + DimMax<E>,
    E: Dimension,
{
    left: Shared<Array<f32, D>>,
    right: Shared<Array<f32, E>>,
    data: Shared<Array<f32, Broadcast<D, E>>>,
}

impl<D, E> Addition<D, E>
where
    D: Dimension + DimMax<E>,
    E: Dimension,
{
    pub(crate) fn new(
        left: Shared<Array<f32, D>>,
        right: Shared<Array<f32, E>>,
        data: Shared<Array<f32, Broadcast<D, E>>>,
    ) -> Self {
        Self { left, right, data }
    }
}

impl<D, E> Forward for Addition<D, E>
where
    D: Dimension + DimMax<E>,
    E: Dimension,
{
    fn forward(&self) {
        Zip::from(&mut *self.data.borrow_mut())
            .and_broadcast(&*self.left.borrow())
            .and_broadcast(&*self.right.borrow())
            .for_each(|v, &l, &r| *v = l + r);
    }
}
pub(crate) struct AdditionBackwardLeft<D, E>
where
    D: Dimension + DimMax<E>,
    E: Dimension,
{
    operand_gradient: Rc<Gradient<D>>,
    gradient: Rc<Gradient<Broadcast<D, E>>>,
}

impl<D, E> AdditionBackwardLeft<D, E>
where
    D: Dimension + DimMax<E>,
    E: Dimension,
{
    pub(crate) fn new(
        operand_gradient: Rc<Gradient<D>>,
        gradient: Rc<Gradient<Broadcast<D, E>>>,
    ) -> Self {
        Self {
            operand_gradient,
            gradient,
        }
    }
}

impl<D, E> Backward for AdditionBackwardLeft<D, E>
where
    D: Dimension + DimMax<E>,
    E: Dimension,
{
    fn backward(&self) {
        let reduced = reduce(self.operand_gradient.shape(), &*self.gradient.borrow());
        *self.operand_gradient.borrow_mut() += &reduced;
    }
}

pub(crate) struct AdditionBackwardRight<D, E>
where
    D: Dimension + DimMax<E>,
    E: Dimension,
{
    operand_gradient: Rc<Gradient<E>>,
    gradient: Rc<Gradient<Broadcast<D, E>>>,
}

impl<D, E> AdditionBackwardRight<D, E>
where
    D: Dimension + DimMax<E>,
    E: Dimension,
{
    pub(crate) fn new(
        operand_gradient: Rc<Gradient<E>>,
        gradient: Rc<Gradient<Broadcast<D, E>>>,
    ) -> Self {
        Self {
            operand_gradient,
            gradient,
        }
    }
}

impl<D, E> Backward for AdditionBackwardRight<D, E>
where
    D: Dimension + DimMax<E>,
    E: Dimension,
{
    fn backward(&self) {
        let reduced = reduce(self.operand_gradient.shape(), &*self.gradient.borrow());
        *self.operand_gradient.borrow_mut() += &reduced;
    }
}

pub(crate) struct AdditionBackward<D, E>
where
    D: Dimension + DimMax<E>,
    E: Dimension,
{
    left: AdditionBackwardLeft<D, E>,
    right: AdditionBackwardRight<D, E>,
}

impl<D, E> AdditionBackward<D, E>
where
    D: Dimension + DimMax<E>,
    E: Dimension,
{
    pub(crate) fn new(
        left: AdditionBackwardLeft<D, E>,
        right: AdditionBackwardRight<D, E>,
    ) -> Self {
        Self { left, right }
    }
}

impl<D, E> Backward for AdditionBackward<D, E>
where
    D: Dimension + DimMax<E>,
    E: Dimension,
{
    fn backward(&self) {
        self.left.backward();
        self.right.backward();
    }
}

// #[cfg(test)]
// mod test;
