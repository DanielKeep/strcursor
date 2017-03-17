pub struct FiniteIter<T, F>(Option<T>, F);

impl<T, F> Iterator for FiniteIter<T, F>
where
    F: FnMut(T) -> Option<T>,
    T: Clone,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.take().and_then(|last| {
            match (self.1)(last) {
                Some(e) => {
                    self.0 = Some(e);
                    self.0.clone()
                },
                None => None
            }
        })
    }
}

pub fn finite_iterate<T, F>(seed: T, f: F) -> FiniteIter<T, F>
where
    F: FnMut(T) -> Option<T>,
    T: Clone,
{
    FiniteIter(Some(seed), f)
}
pub struct FiniteIterLead<T, F>(Option<T>, F, bool);

impl<T, F> Iterator for FiniteIterLead<T, F>
where
    F: FnMut(T) -> Option<T>,
    T: Clone,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.2 {
            self.2 = true;
            return self.0.clone();
        }

        self.0.take().and_then(|last| {
            match (self.1)(last) {
                Some(e) => {
                    self.0 = Some(e);
                    self.0.clone()
                },
                None => None
            }
        })
    }
}

pub fn finite_iterate_lead<T, F>(seed: T, f: F) -> FiniteIterLead<T, F>
where
    F: FnMut(T) -> Option<T>,
    T: Clone,
{
    FiniteIterLead(Some(seed), f, false)
}
