#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

/// Figure 11.16: Using a barrier
///
/// Finding about runtime speed:
///
/// - 8 threads are really faster than 1 thread
/// - building without --release is ten times slower than when using --release
///   see my question on stackoverflow: http://stackoverflow.com/questions/42688721
/// - XorShiftRng is about 2x as fast as random() from libc
///
/// On my 4 core computer (hyperthreaded so it's 8 logical cores):
///
///            1 Thread   4 Threads   8 Threads
/// Debug      3.19s      2.45s       3.48s
/// Release    1.25s      0.41s       0.35s
///
/// Strange is that the debug version takes considerably longer
/// with 8 threads than with 4 threads.
///
/// Other findings:
/// - this time the book did not say at all that OSX does not implement
///   pthread_barrier_*, needed to take a C implementation I found on the
///   web
/// - the mutable statics are ugly, a bit nicer would have been to pass
///   a struct to thr_fn..
/// - merge() is really hard to understand, I guess that's typical C
///   code. Performant but hard to grasp..
///
/// $ f16-barrier | sed 's/[\.0-9]*//g'
/// sort took  seconds

extern crate libc;
extern crate rand;
extern crate apue;

use apue::my_libc::{qsort, pthread_create};
use libc::{c_long, c_void, c_int, c_uint, pthread_mutex_t, pthread_cond_t,
           PTHREAD_MUTEX_INITIALIZER, PTHREAD_COND_INITIALIZER};
use libc::gettimeofday;
use std::ptr::{null, null_mut};
use std::mem::{uninitialized, size_of};
use rand::Rng;

const NTHR: usize = 8;
const NUMNUM: usize = 8_000_000;
const TNUM: usize = NUMNUM / NTHR;

pub type pthread_barrierattr_t = c_int;
#[repr(C)]
pub struct pthread_barrier_t {
    pub mutex: pthread_mutex_t,
    pub cond: pthread_cond_t,
    pub count: c_int,
    pub tripCount: c_int,
}

static mut B: pthread_barrier_t = pthread_barrier_t {
    mutex: PTHREAD_MUTEX_INITIALIZER,
    cond: PTHREAD_COND_INITIALIZER,
    count: 0,
    tripCount: 0,
};
static mut NUMS: [c_long; NUMNUM] = [0; NUMNUM];

extern "C" {
    pub fn pthread_barrier_init(barrier: *mut pthread_barrier_t,
                                attr: *const pthread_barrierattr_t,
                                count: c_uint)
                                -> c_int;
    pub fn pthread_barrier_destroy(barrier: *mut pthread_barrier_t) -> c_int;
    pub fn pthread_barrier_wait(barrier: *mut pthread_barrier_t) -> c_int;
}

unsafe extern "C" fn thr_fn(arg: *mut c_void) -> *mut c_void {
    let idx: c_long = arg as c_long;
    qsort(NUMS.as_mut_ptr().offset(idx as isize) as _,
          TNUM,
          size_of::<c_long>(),
          cmp);
    pthread_barrier_wait(&mut B);
    0 as *mut c_void
}

extern "C" fn cmp(val1: *const c_void, val2: *const c_void) -> c_int {
    unsafe {
        let val1 = val1 as *const c_long;
        let val2 = val2 as *const c_long;
        if *val1 == *val2 {
            0
        } else if *val1 < *val2 {
            -1
        } else {
            1
        }
    }
}

unsafe fn merge() -> Vec<c_long> {
    let mut idx = [0usize; NTHR];
    let mut snums = Vec::with_capacity(NUMNUM);
    for i in 0..NTHR {
        idx[i] = i * TNUM;
    }
    for _ in 0..NUMNUM {
        let mut num = c_long::max_value();
        let mut minidx = 0;
        for i in 0..NTHR {
            if idx[i] < (i + 1) * TNUM && NUMS[idx[i]] < num {
                num = NUMS[idx[i]];
                minidx = i;
            }
        }
        snums.push(NUMS[idx[minidx]]);
        idx[minidx] += 1;
    }
    snums
}

fn main() {
    unsafe {
        let (mut tid, mut start, mut end) = uninitialized();

        let mut rng = rand::XorShiftRng::new_unseeded();
        for i in 0..NUMNUM - 1 {
            NUMS[i] = rng.gen();
        }
        // create 8 threads to sort the numbers
        gettimeofday(&mut start, null_mut());
        // barrier count = num worker threads + 1 because main thread counts as 1 waiter
        pthread_barrier_init(&mut B, null(), (NTHR + 1) as _);
        for i in 0..NTHR {
            let err = pthread_create(&mut tid, null_mut(), thr_fn, (i * TNUM) as *mut c_void);
            if err != 0 {
                panic!("can't create thread, error: {}", err)
            }
        }
        pthread_barrier_wait(&mut B);
        let res = merge();
        gettimeofday(&mut end, null_mut());
        let startusec = start.tv_sec * 1_000_000 + start.tv_usec as i64;
        let endusec = end.tv_sec * 1_000_000 + end.tv_usec as i64;
        let elapsed = (endusec - startusec) as f64 / 1_000_000f64;
        println!("sort took {} seconds", elapsed);
        let mut pre = c_long::min_value();
        for n in res {
            assert!(pre <= n);
            pre = n;
        }
    }
}
