/// Find the minimum cut where the sum of elements size on the right-hand
/// side of the cut is greater or equal to `size`.
///
/// This function iterates `values` in reverse order.
/// Each element `element_size` is computed and accumulated in a total size.
/// As soon as the total accumulated size is greater or equal to `size`,
/// the index of the last counted element, the total accumulated size and
/// a reference to the element on the cut are respectively returned.
///
/// If the input is empty, (0,0,None) is returned.
pub fn find_cut_at_size<'a, T, S, I, F>(
    values: &'a S,
    element_size: F,
    size: usize,
) -> (usize, usize, Option<&T>)
where
    F: Fn(&T) -> usize,
    T: 'a,
    &'a S: 'a + IntoIterator<IntoIter = I>,
    I: 'a
        + Iterator<Item = &'a T>
        + DoubleEndedIterator
        + ExactSizeIterator,
{
    match values
        .into_iter()
        .map(|e| ((element_size)(e), e))
        .enumerate()
        .rev()
        .try_fold(
            (0usize, (0usize, None)),
            |(_, (acc, _)), (index, (s, e))| {
                let total_size = acc + s;
                if total_size < size {
                    Ok((index, (total_size, Some(e)))) // Continue iteration
                } else {
                    Err((index, (total_size, Some(e)))) // Stop iteration
                }
            },
        ) {
        Ok((i, (size, element))) => (i, size, element),
        Err((i, (size, element))) => (i, size, element),
    }
}
