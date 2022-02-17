use ndarray::Array1;

/*
pub fn norm_entropy(arr: Array1<i32>, n: i32) -> f32 {
    let n = n as f32;
    let arr = arr
        .into_iter()
        .filter(|&x| x > 1)
        .map(|x| x as f32)
        .collect::<Array1<f32>>();

    let arr_log = {
        let mut arr_log = arr.clone();
        arr_log.par_mapv_inplace(|x| x.log(n));
        arr_log
    };

    1. - 1. / (n) * (arr * arr_log).sum()
}
*/

pub fn entropy(arr: Array1<f32>) -> f32 {
    let arr = arr
        .into_iter()
        .filter(|&x| x > 0.)
        .map(|x| x as f32)
        .collect::<Array1<f32>>();

    let arr_log = {
        let mut arr_log = arr.clone();
        arr_log.par_mapv_inplace(|x| (x).log2());
        arr_log
    };

    - 1. * (arr * arr_log).sum()
}