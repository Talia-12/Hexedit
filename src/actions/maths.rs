use itertools::Either::{Left, Right};

use crate::simulator::{Iota, ConstLenAction, ActionError};

pub struct Add;

impl ConstLenAction for Add {
    fn len() -> usize { 2 }

    fn apply(&self, iotas: &[Iota]) -> Vec<Result<Vec<Iota>, ActionError>> {
			vec![
			match iotas[0] {
        Iota::Double(d0) => {
					if let Some(d0) = d0 {
						match iotas[1] {
							Iota::Double(d1) => Ok(vec![d1.map(|d1| d0 + d1).into()]),
							Iota::Vec(vec1) => Ok(vec![vec1.map_left(|vec1| (vec1.0 + d0, vec1.1 + d0, vec1.2 + d0)).into()]),
							_ => Err(ActionError::InvalidType),
						}
					} else {
						Ok(vec![Iota::Double(None)])
					}
				}
        Iota::Vec(vec0) => {
					match vec0 {
						Left(vec0) => match iotas[1] {
							Iota::Double(d1) => Ok(vec![ if let Some(d1) = d1 { (vec0.0 + d1, vec0.1 + d1, vec0.2 + d1).into() } else { Right(false).into() } ]),
							Iota::Vec(_) => todo!(),
							_ => Err(ActionError::InvalidType),
						},
						Right(_) => Ok(vec![vec0.into()]),
					}
				},
        _ => Err(ActionError::InvalidType),
    	}
			]
    }
}