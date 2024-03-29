use super::{
    assert_almost_equals, new_backward_input, new_input, new_tensor, Backward, Cache, Data,
    Forward, Gradient, MatrixVectorMul, MatrixVectorMulBackward, MatrixVectorMulBackwardLeft,
    MatrixVectorMulBackwardRight, Overwrite, Tensor,
};

#[cfg(feature = "blas")]
extern crate blas_src;

mod forward {
    use super::{
        assert_almost_equals, new_input, new_tensor, Cache, Data, Forward, MatrixVectorMul, Tensor,
    };

    #[test]
    fn creation() {
        let left = new_input((3, 3), vec![1., 2., 3., 4., 5., 6., 7., 8., 9.]);
        let right = new_input(3, vec![1.; 3]);
        let node = MatrixVectorMul::new(left, right);

        assert_eq!(*node.data(), Tensor::from_elem(3, 0.));
        assert_eq!(*node.data_mut(), Tensor::from_elem(3, 0.));
        assert!(!node.was_computed());
    }

    #[test]
    fn computation_was_computed_transition() {
        let left = new_input((3, 3), vec![1., 2., 3., 4., 5., 6., 7., 8., 9.]);
        let right = new_input(3, vec![1.; 3]);
        let node = MatrixVectorMul::new(left, right);

        node.forward();
        assert!(node.was_computed());

        node.forward();
        assert!(node.was_computed());

        node.reset_computation();
        assert!(!node.was_computed());

        node.reset_computation();
        assert!(!node.was_computed());
    }

    #[test]
    fn forward() {
        let left = new_input((3, 3), vec![1., 2., 3., 4., 5., 6., 7., 8., 9.]);
        let right = new_input(3, vec![1.; 3]);
        let node = MatrixVectorMul::new(left, right.clone());

        // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ First Evaluation ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        node.forward();
        assert_almost_equals(&*node.data(), &new_tensor(3, vec![6., 15., 24.]));

        // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ No Second Evaluation ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        *right.data_mut() = new_tensor(3, vec![-2.; 3]);
        assert_almost_equals(&*right.data(), &new_tensor(3, vec![-2.; 3]));

        node.forward();
        assert_almost_equals(&*node.data(), &new_tensor(3, vec![6., 15., 24.]));

        // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ Second Evaluation ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        node.reset_computation();
        node.forward();
        assert_almost_equals(&*node.data(), &new_tensor(3, vec![-12., -30., -48.]));
    }

    #[test]
    fn debug() {
        let left = new_input((3, 3), vec![1., 2., 3., 4., 5., 6., 7., 8., 9.]);
        let right = new_input(3, vec![1.; 3]);
        let node = MatrixVectorMul::new(left, right.clone());

        let output = "MatrixVectorMul { data: [0.0, 0.0, 0.0], shape=[3], strides=[1], layout=CFcf (0xf), const ndim=1, computed: false }";

        assert_eq!(output, format!("{:?}", node));
    }

    #[test]
    fn display() {
        let left = new_input((3, 3), vec![1., 2., 3., 4., 5., 6., 7., 8., 9.]);
        let right = new_input(3, vec![1.; 3]);
        let node = MatrixVectorMul::new(left, right.clone());

        assert_eq!(format!("{}", node.data()), format!("{}", node));
    }
}

mod backward {
    use super::{
        assert_almost_equals, new_backward_input, new_input, new_tensor, Backward, Gradient,
        MatrixVectorMulBackward, MatrixVectorMulBackwardLeft, MatrixVectorMulBackwardRight,
        Overwrite, Tensor,
    };

    #[test]
    fn creation() {
        let node = MatrixVectorMulBackward::new(
            new_input((3, 3), vec![1., 2., 3., 4., 5., 6., 7., 8., 9.]),
            new_backward_input((3, 3), vec![0.; 9]),
            new_input(3, vec![1., 2., 3.]),
            new_backward_input(3, vec![0.; 3]),
        );

        assert_eq!(*node.gradient(), Tensor::from_elem(3, 0.));
        assert_eq!(*node.gradient_mut(), Tensor::from_elem(3, 0.));
        assert!(node.can_overwrite());
    }

    #[test]
    fn computation_state_transition() {
        let lhs = new_backward_input((3, 3), vec![0.; 9]);
        let rhs = new_backward_input(3, vec![0.; 3]);
        let node = MatrixVectorMulBackward::new(
            new_input((3, 3), vec![1., 2., 3., 4., 5., 6., 7., 8., 9.]),
            lhs.clone(),
            new_input(3, vec![1., 2., 3.]),
            rhs.clone(),
        );

        node.backward();
        assert!(node.can_overwrite());
        assert!(!lhs.can_overwrite());
        assert!(!rhs.can_overwrite());

        node.backward();
        assert!(node.can_overwrite());
        assert!(!lhs.can_overwrite());
        assert!(!rhs.can_overwrite());

        lhs.set_overwrite(true);
        assert!(node.can_overwrite());
        assert!(lhs.can_overwrite());
        assert!(!rhs.can_overwrite());

        lhs.set_overwrite(true);
        assert!(node.can_overwrite());
        assert!(lhs.can_overwrite());
        assert!(!rhs.can_overwrite());

        rhs.set_overwrite(true);
        assert!(node.can_overwrite());
        assert!(lhs.can_overwrite());
        assert!(rhs.can_overwrite());

        rhs.set_overwrite(true);
        assert!(node.can_overwrite());
        assert!(lhs.can_overwrite());
        assert!(rhs.can_overwrite());

        node.set_overwrite(false);
        assert!(!node.can_overwrite());
        assert!(lhs.can_overwrite());
        assert!(rhs.can_overwrite());

        node.set_overwrite(false);
        assert!(!node.can_overwrite());
        assert!(lhs.can_overwrite());
        assert!(rhs.can_overwrite());

        node.backward();
        assert!(!node.can_overwrite());
        assert!(!lhs.can_overwrite());
        assert!(!rhs.can_overwrite());

        node.backward();
        assert!(!node.can_overwrite());
        assert!(!lhs.can_overwrite());
        assert!(!rhs.can_overwrite());
    }

    #[test]
    fn backward() {
        let lhs = new_backward_input((3, 3), vec![0.; 9]);
        let rhs = new_backward_input(3, vec![0.; 3]);
        let node = MatrixVectorMulBackward::new(
            new_input((3, 3), vec![1., 2., 3., 4., 5., 6., 7., 8., 9.]),
            lhs.clone(),
            new_input(3, vec![1., 2., 3.]),
            rhs.clone(),
        );

        // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ Seed Gradient ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        *node.gradient_mut() = new_tensor(3, vec![1.; 3]);
        assert_almost_equals(&*node.gradient(), &new_tensor(3, vec![1.; 3]));

        // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ First Evaluation ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        node.backward();
        assert_almost_equals(
            &*lhs.gradient(),
            &new_tensor((3, 3), vec![1., 2., 3., 1., 2., 3., 1., 2., 3.]),
        );
        assert_almost_equals(&*rhs.gradient(), &new_tensor(3, vec![12., 15., 18.]));

        // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ Second Evaluation ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        node.backward();
        assert_almost_equals(
            &*lhs.gradient(),
            &new_tensor((3, 3), vec![2., 4., 6., 2., 4., 6., 2., 4., 6.]),
        );
        assert_almost_equals(&*rhs.gradient(), &new_tensor(3, vec![24., 30., 36.]));

        // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ Third Evaluation ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        lhs.set_overwrite(true);
        rhs.set_overwrite(true);
        node.backward();
        assert_almost_equals(
            &*lhs.gradient(),
            &new_tensor((3, 3), vec![1., 2., 3., 1., 2., 3., 1., 2., 3.]),
        );
        assert_almost_equals(&*rhs.gradient(), &new_tensor(3, vec![12., 15., 18.]));
    }

    #[test]
    fn debug() {
        let lhs = new_backward_input((3, 3), vec![0.; 9]);
        let rhs = new_backward_input(3, vec![0.; 3]);
        let node = MatrixVectorMulBackward::new(
            new_input((3, 3), vec![1., 2., 3., 4., 5., 6., 7., 8., 9.]),
            lhs,
            new_input(3, vec![1., 2., 3.]),
            rhs,
        );

        let output = "MatrixVectorMulBackward { gradient: Some([0.0, 0.0, 0.0], shape=[3], strides=[1], layout=CFcf (0xf), const ndim=1), overwrite: true }";

        assert_eq!(output, format!("{:?}", node));
    }

    #[test]
    fn display() {
        let lhs = new_backward_input((3, 3), vec![0.; 9]);
        let rhs = new_backward_input(3, vec![0.; 3]);
        let node = MatrixVectorMulBackward::new(
            new_input((3, 3), vec![1., 2., 3., 4., 5., 6., 7., 8., 9.]),
            lhs,
            new_input(3, vec![1., 2., 3.]),
            rhs,
        );

        assert_eq!(format!("{}", node.gradient()), format!("{}", node));
    }

    #[test]
    fn backward_left() {
        let diff = new_backward_input((3, 3), vec![0.; 9]);
        let node = MatrixVectorMulBackwardLeft::new(diff.clone(), new_input(3, vec![1., 2., 3.]));

        // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ Seed Gradient ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        *node.gradient_mut() = new_tensor(3, vec![1.; 3]);
        assert_almost_equals(&*node.gradient(), &new_tensor(3, vec![1.; 3]));

        // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ First Evaluation ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        node.backward();
        assert_almost_equals(
            &*diff.gradient(),
            &new_tensor((3, 3), vec![1., 2., 3., 1., 2., 3., 1., 2., 3.]),
        );

        // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ Second Evaluation ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        node.backward();
        assert_almost_equals(
            &*diff.gradient(),
            &new_tensor((3, 3), vec![2., 4., 6., 2., 4., 6., 2., 4., 6.]),
        );

        // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ Third Evaluation ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        diff.set_overwrite(true);
        node.backward();
        assert_almost_equals(
            &*diff.gradient(),
            &new_tensor((3, 3), vec![1., 2., 3., 1., 2., 3., 1., 2., 3.]),
        );
    }

    #[test]
    fn debug_left() {
        let diff = new_backward_input((3, 3), vec![0.; 9]);
        let node = MatrixVectorMulBackwardLeft::new(diff.clone(), new_input(3, vec![1., 2., 3.]));

        let output = "MatrixVectorMulBackwardLeft { gradient: Some([0.0, 0.0, 0.0], shape=[3], strides=[1], layout=CFcf (0xf), const ndim=1), overwrite: true }";

        assert_eq!(output, format!("{:?}", node));
    }

    #[test]
    fn display_left() {
        let diff = new_backward_input((3, 3), vec![0.; 9]);
        let node = MatrixVectorMulBackwardLeft::new(diff.clone(), new_input(3, vec![1., 2., 3.]));

        assert_eq!(format!("{}", node.gradient()), format!("{}", node));
    }

    #[test]
    fn backward_right() {
        let diff = new_backward_input(3, vec![0.; 3]);
        let node = MatrixVectorMulBackwardRight::new(
            new_input((3, 3), vec![1., 2., 3., 4., 5., 6., 7., 8., 9.]),
            diff.clone(),
        );

        // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ Seed Gradient ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        *node.gradient_mut() = new_tensor(3, vec![1.; 3]);
        assert_almost_equals(&*node.gradient(), &new_tensor(3, vec![1.; 3]));

        // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ First Evaluation ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        node.backward();
        assert_almost_equals(&*diff.gradient(), &new_tensor(3, vec![12., 15., 18.]));

        // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ Second Evaluation ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        node.backward();
        assert_almost_equals(&*diff.gradient(), &new_tensor(3, vec![24., 30., 36.]));

        // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ Third Evaluation ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        diff.set_overwrite(true);
        node.backward();
        assert_almost_equals(&*diff.gradient(), &new_tensor(3, vec![12., 15., 18.]));
    }

    #[test]
    fn debug_right() {
        let diff = new_backward_input(3, vec![0.; 3]);
        let node = MatrixVectorMulBackwardRight::new(
            new_input((3, 3), vec![1., 2., 3., 4., 5., 6., 7., 8., 9.]),
            diff,
        );

        let output = "MatrixVectorMulBackwardRight { gradient: Some([0.0, 0.0, 0.0], shape=[3], strides=[1], layout=CFcf (0xf), const ndim=1), overwrite: true }";

        assert_eq!(output, format!("{:?}", node));
    }

    #[test]
    fn display_right() {
        let diff = new_backward_input(3, vec![0.; 3]);
        let node = MatrixVectorMulBackwardRight::new(
            new_input((3, 3), vec![1., 2., 3., 4., 5., 6., 7., 8., 9.]),
            diff.clone(),
        );

        assert_eq!(format!("{}", node.gradient()), format!("{}", node));
    }

    #[test]
    fn no_grad() {
        // MatrixVectorMulBackward
        let node = MatrixVectorMulBackward::new(
            new_input((3, 3), vec![0.; 9]),
            new_backward_input((3, 3), vec![0.; 9]),
            new_input(3, vec![0.; 3]),
            new_backward_input(3, vec![0.; 3]),
        );

        node.no_grad();
        assert!(node.gradient.borrow().is_none());

        node.with_grad();
        assert_eq!(&*node.gradient(), Tensor::zeros(node.shape));

        // MatrixVectorMulBackwardLeft
        let node = MatrixVectorMulBackwardLeft::new(
            new_backward_input((3, 3), vec![0.; 9]),
            new_input(3, vec![0.; 3]),
        );

        node.no_grad();
        assert!(node.gradient.borrow().is_none());

        node.with_grad();
        assert_eq!(&*node.gradient(), Tensor::zeros(node.shape));

        // MatrixVectorMulBackwardRight
        let node = MatrixVectorMulBackwardRight::new(
            new_input((3, 3), vec![0.; 9]),
            new_backward_input(3, vec![0.; 3]),
        );

        node.no_grad();
        assert!(node.gradient.borrow().is_none());

        node.with_grad();
        assert_eq!(&*node.gradient(), Tensor::zeros(node.shape));
    }
}
