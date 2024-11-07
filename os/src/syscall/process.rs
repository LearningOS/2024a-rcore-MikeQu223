//! Process management syscalls
use crate::{
    config::PAGE_SIZE,
    mm::{translated_byte_buffer, MapPermission, VPNRangeOuter, VirtAddr, VirtPageNum},
    task::{
        change_program_brk, check_vpn_exists, current_user_token, do_mmap, do_munmap, exit_current_and_run_next, get_task_info, suspend_current_and_run_next, TaskInfo
    },
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

// copy to user space
pub fn copy_to_user_space<T>(dest: *mut T, src: &T) {
    let size = core::mem::size_of::<T>();
    let src_ptr = src as *const T as *const u8;
    let dest_ptr = dest as *mut T as *mut u8;

    let tar = translated_byte_buffer(current_user_token(), dest_ptr, size);

    for (i, dst) in tar.into_iter().enumerate() {
        unsafe {
            core::ptr::copy_nonoverlapping(src_ptr.add(i), dst.as_mut_ptr(), dst.len());
        }
    }
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    if _ts.is_null() {
        error!("sys_get_time: ts is null");
        return -1;
    }
    let time_us = get_time_us();
    let time_val = TimeVal {
        sec: time_us / 1_000_000,
        usec: time_us % 1_000_000,
    };
    copy_to_user_space(_ts, &time_val);
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");
    let task_info = get_task_info();
    copy_to_user_space(_ti, &task_info);
    0
}

// 校验和翻译port
fn check_and_translate_port(port: usize) -> Option<MapPermission> {
    if port & !0x7 != 0 {
        // 其他位必须为0
        return None;
    }
    if port & 0x7 == 0 {
        // 至少要有一位有效
        return None;
    }

    // 翻译
    let mut flags = MapPermission::U; // 用户态
    if port & 0x1 != 0 {
        flags |= MapPermission::R;
    } // 可读
    if port & 0x2 != 0 {
        flags |= MapPermission::W;
    } // 可写
    if port & 0x4 != 0 {
        flags |= MapPermission::X;
    } // 可执行
    Some(flags)
}

pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap");

    // 检查页对齐和非法长度
    if start % PAGE_SIZE != 0 {
        return -1;
    }
    if len == 0 {
        return -1;
    }

    // 翻译port
    let perm = match check_and_translate_port(port) {
        Some(flags) => flags,
        None => return -1,
    };

    // 检查虚拟地址是否已经被映射
    let end = start + len;
    let s_vpn: VirtPageNum = VirtAddr::from(start).floor().into();
    let e_vpn: VirtPageNum = VirtAddr::from(end).ceil().into();
    for cur_vpn in VPNRangeOuter::new(s_vpn, e_vpn) {
        info!("range: {start} to {end}, cur_vpn: {:?}", cur_vpn);
        if check_vpn_exists(cur_vpn) {
            return -1;
        }
    }

    // do_mmap
    if !do_mmap(start, len, perm) {
        return -1;
    }

    0
}


pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap");

    // 检查页对齐和非法长度
    if start % PAGE_SIZE != 0 {
        return -1;
    }
    if len == 0 {
        return -1;
    }

    // 计算虚拟地址范围
    let end = start + len;
    let s_vpn: VirtPageNum = VirtAddr::from(start).floor().into();
    let e_vpn: VirtPageNum = VirtAddr::from(end).ceil().into();

    // 检查已经被映射
    for cur_vpn in VPNRangeOuter::new(s_vpn, e_vpn) {
        if !check_vpn_exists(cur_vpn) {
            return -1; 
        }
        // do_munmap
        if !do_munmap(cur_vpn) {
            return -1;
        }
    }

    0
}

/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
