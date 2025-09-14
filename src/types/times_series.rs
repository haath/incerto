pub struct TimeSeries<'a, T>
{
    pub(crate) values: Vec<&'a T>,
    pub(crate) sample_interval: usize,
}

impl<T> TimeSeries<'_, T>
{
    /// The number of samples in the time series.
    ///
    /// Not that this does not represent the length of the series across time.
    /// To obtain the temoral length of the series use [`Self::duration`].
    #[must_use]
    pub const fn len(&self) -> usize
    {
        self.values.len()
    }

    /// Returns `true` if the series has no samples.
    #[must_use]
    pub const fn is_empty(&self) -> bool
    {
        self.values.is_empty()
    }

    /// Returns the length of the time series across time.
    ///
    /// The value is in number of simulation steps.
    #[must_use]
    pub const fn duration(&self) -> usize
    {
        self.values.len() * self.sample_interval
    }

    /// Returns the sample interval with which this time series was sampled.
    ///
    /// The value is in number of simulation steps.
    #[must_use]
    pub const fn sample_interval(&self) -> usize
    {
        self.sample_interval
    }

    /// Iterates over the time range in which this series was sampled.
    ///
    /// With a sample interval `s`, this will always produce the following sequence:
    /// { `0`, `1s`, `2s`, `3s`, .. }
    pub fn time_range(&self) -> impl Iterator<Item = usize>
    {
        (0..self.values.len()).map(|t| t * self.sample_interval)
    }

    /// Iterates over the values in the time series.
    pub fn values(&self) -> impl Iterator<Item = &T>
    {
        self.values.iter().copied()
    }

    /// Iterates over each time-value point in the time series.
    pub fn enumerate(&self) -> impl Iterator<Item = (usize, &T)>
    {
        self.time_range().zip(self.values())
    }
}

impl<T> TimeSeries<'_, T>
where
    T: Copy,
{
    /// Iterates over the values in the time series.
    pub fn values_copied(&self) -> impl Iterator<Item = T>
    {
        self.values().copied()
    }

    /// Iterates over each time-value point in the time series.
    pub fn enumerate_copied(&self) -> impl Iterator<Item = (usize, T)>
    {
        self.time_range().zip(self.values_copied())
    }
}
