use std::ffi::OsString;
use std::fmt::{Debug, Formatter};
use std::os::{raw::c_void, windows::ffi::OsStringExt};
use std::path::PathBuf;
use winapi::shared::minwindef::FALSE;
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::winbase::QueryFullProcessImageNameW;
use winapi::um::winnt::PROCESS_QUERY_LIMITED_INFORMATION;

use crate::os::win::internal::ntdll::{NtQuerySystemInformation, SystemProcessInformation};
use crate::os::win::internal::ntdll::{SYSTEM_PROCESS_INFORMATION, SYSTEM_THREAD_INFORMATION};
use crate::os::win::WindowsHandle;

impl Process {
    pub fn current() -> Vec<Process> {
        unimplemented!();
    }

    pub fn get() -> Vec<Process> {
        Process::enumerate().into_iter().collect()
    }

    pub fn enumerate() -> ProcessCollection {
        let data = unsafe {
            let mut data = vec![];
            let mut length = 0;

            let status = NtQuerySystemInformation(
                SystemProcessInformation,
                data.as_mut_ptr() as *mut c_void,
                data.capacity() as u32,
                &mut length,
            );

            while status < 0 {
                let length = length as usize;

                if data.capacity() < length {
                    data.reserve_exact(length - data.len());
                }
            }

            data.set_len(length as usize);
            data
        };

        ProcessCollection { data }
    }
}

#[derive(Clone)]
pub struct ProcessCollection {
    data: Vec<u8>,
}

impl ProcessCollection {
    pub fn iter<'a>(&'a self) -> ProcessCollectionIterator<'a> {
        ProcessCollectionIterator::new(&self.data)
    }
}

impl IntoIterator for ProcessCollection {
    type IntoIter = ProcessCollectionOwnedIterator;
    type Item = Process;

    fn into_iter(self) -> ProcessCollectionOwnedIterator {
        ProcessCollectionOwnedIterator::new(self.data)
    }
}

impl Debug for ProcessCollection {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), std::fmt::Error> {
        formatter.debug_list().entries(self.iter()).finish()
    }
}

pub struct ProcessCollectionOwnedIterator {
    data: Vec<u8>,
    index: usize,
}

impl ProcessCollectionOwnedIterator {
    fn new(data: Vec<u8>) -> ProcessCollectionOwnedIterator {
        ProcessCollectionOwnedIterator { data, index: 0 }
    }
}

impl Iterator for ProcessCollectionOwnedIterator {
    type Item = Process;

    fn next(&mut self) -> Option<Process> {
        let data = &self.data[self.index..];

        if let Some((length, process, threads)) = IteratorImpl::step(data) {
            let process = process.to_owned();
            let threads = threads.to_owned();

            self.index = std::cmp::min(self.index + length, self.data.len());

            Some(Process::new(process, threads))
        } else {
            None
        }
    }
}

pub struct ProcessCollectionIterator<'a> {
    data: &'a [u8],
}

impl ProcessCollectionIterator<'_> {
    fn new(data: &[u8]) -> ProcessCollectionIterator {
        ProcessCollectionIterator { data }
    }
}

impl<'a> Iterator for ProcessCollectionIterator<'a> {
    type Item = ProcessS<'a>;

    fn next(&mut self) -> Option<ProcessS<'a>> {
        if let Some((length, process, threads)) = IteratorImpl::step(self.data) {
            self.data = &self.data[length..];

            Some(ProcessS::new(process, threads))
        } else {
            None
        }
    }
}

struct IteratorImpl;

impl IteratorImpl {
    fn step(
        data: &[u8],
    ) -> Option<(
        usize,
        &SYSTEM_PROCESS_INFORMATION,
        &[SYSTEM_THREAD_INFORMATION],
    )> {
        if data.is_empty() {
            None
        } else {
            unsafe {
                let process = data.as_ptr() as *const SYSTEM_PROCESS_INFORMATION;
                let process = &*process;

                let offset = process.NextEntryOffset as usize;
                let length = if offset > 0 { offset } else { data.len() };
                let threads = std::slice::from_raw_parts(
                    process as *const _ as *const SYSTEM_THREAD_INFORMATION,
                    process.NumberOfThreads as usize,
                );

                Some((length, process, threads))
            }
        }
    }
}

#[derive(Clone)]
pub struct Process {
    process: SYSTEM_PROCESS_INFORMATION,
    threads: Vec<SYSTEM_THREAD_INFORMATION>,
}

impl Process {
    fn new(
        process: SYSTEM_PROCESS_INFORMATION,
        threads: Vec<SYSTEM_THREAD_INFORMATION>,
    ) -> Process {
        Process { process, threads }
    }

    pub fn id(&self) -> u32 {
        ProcessImpl::id(&self.process)
    }

    pub fn name(&self) -> OsString {
        ProcessImpl::name(&self.process)
    }

    pub fn wide_name(&self) -> &[u16] {
        ProcessImpl::wide_name(&self.process)
    }

    pub fn location(&self) -> Result<PathBuf, std::io::Error> {
        ProcessImpl::location(&self.process)
    }

    pub fn threads(&self) -> impl Iterator<Item = ThreadS<'_>> {
        ProcessImpl::threads(&self.threads)
    }
}

impl Debug for Process {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), std::fmt::Error> {
        ProcessImpl::fmt("Process", &self.process, &self.threads, formatter)
    }
}

#[derive(Clone)]
pub struct ProcessS<'a> {
    process: &'a SYSTEM_PROCESS_INFORMATION,
    threads: &'a [SYSTEM_THREAD_INFORMATION],
}

impl<'a> ProcessS<'a> {
    fn new(
        process: &'a SYSTEM_PROCESS_INFORMATION,
        threads: &'a [SYSTEM_THREAD_INFORMATION],
    ) -> ProcessS<'a> {
        ProcessS { process, threads }
    }

    pub fn id(&self) -> u32 {
        ProcessImpl::id(self.process)
    }

    pub fn name(&self) -> OsString {
        ProcessImpl::name(self.process)
    }

    pub fn wide_name(&self) -> &[u16] {
        ProcessImpl::wide_name(self.process)
    }

    pub fn location(&self) -> Result<PathBuf, std::io::Error> {
        ProcessImpl::location(self.process)
    }

    pub fn threads(&self) -> impl Iterator<Item = ThreadS<'_>> {
        ProcessImpl::threads(self.threads)
    }
}

impl Debug for ProcessS<'_> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), std::fmt::Error> {
        ProcessImpl::fmt("ProcessS", self.process, self.threads, formatter)
    }
}

impl Into<Process> for ProcessS<'_> {
    fn into(self) -> Process {
        Process::new(self.process.clone(), self.threads.to_owned())
    }
}

struct ProcessImpl;

impl ProcessImpl {
    fn id(x: &SYSTEM_PROCESS_INFORMATION) -> u32 {
        x.UniqueProcessId as u32
    }

    fn wide_name(x: &SYSTEM_PROCESS_INFORMATION) -> &[u16] {
        unsafe {
            let pointer = x.ImageName.Buffer;
            let length = x.ImageName.Length as usize / 2;

            match length == 0 {
                true => &[],
                false => std::slice::from_raw_parts(pointer, length),
            }
        }
    }

    fn name(x: &SYSTEM_PROCESS_INFORMATION) -> OsString {
        OsString::from_wide(ProcessImpl::wide_name(x))
    }

    fn location(x: &SYSTEM_PROCESS_INFORMATION) -> Result<PathBuf, std::io::Error> {
        // flags for `QueryFullProcessImageName`:
        //     <NONE>              (0x00) => C:\Windows\System32\notepad.exe
        //     PROCESS_NAME_NATIVE (0x01) => \Device\HarddiskVolume3\Windows\System32\notepad.exe
        //
        // const PROCESS_NAME_NATIVE: u32 = 1;

        unsafe {
            let process_id = ProcessImpl::id(x);
            let handle = WindowsHandle::create(
                || OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, process_id),
                |x| !x.is_null(),
            )?;

            let mut data = [0u16; 1024];
            let mut length = data.len() as u32;

            match QueryFullProcessImageNameW(handle.as_raw(), 0, data.as_mut_ptr(), &mut length)
                != 0
            {
                true => Ok(OsString::from_wide(&data[..length as usize]).into()),
                false => Err(std::io::Error::last_os_error()),
            }
        }
    }

    fn threads(x: &[SYSTEM_THREAD_INFORMATION]) -> impl Iterator<Item = ThreadS<'_>> {
        x.iter().map(ThreadS::new)
    }

    fn fmt(
        name: &str,
        process: &SYSTEM_PROCESS_INFORMATION,
        threads: &[SYSTEM_THREAD_INFORMATION],
        formatter: &mut Formatter,
    ) -> Result<(), std::fmt::Error> {
        formatter
            .debug_struct(name)
            .field("id", &ProcessImpl::id(process))
            .field("name", &ProcessImpl::name(process))
            .field("location", &ProcessImpl::location(process))
            .field("threads", &threads.len())
            .finish()
    }
}

#[derive(Clone)]
pub struct Thread {
    data: SYSTEM_THREAD_INFORMATION,
}

impl Thread {
    fn new(data: SYSTEM_THREAD_INFORMATION) -> Thread {
        Thread { data }
    }

    pub fn thread_id(&self) -> u32 {
        ThreadImpl::thread_id(&self.data)
    }

    pub fn process_id(&self) -> u32 {
        ThreadImpl::process_id(&self.data)
    }

    pub fn start_address(&self) -> usize {
        ThreadImpl::start_address(&self.data)
    }

    pub fn thread_state(&self) -> ThreadState {
        ThreadImpl::thread_state(&self.data)
    }

    pub fn wait_reason(&self) -> ThreadWaitReason {
        ThreadImpl::wait_reason(&self.data)
    }

    pub fn priority(&self) -> u32 {
        ThreadImpl::priority(&self.data)
    }

    pub fn base_priority(&self) -> u32 {
        ThreadImpl::base_priority(&self.data)
    }

    pub fn kernel_time(&self) -> () {
        ThreadImpl::kernel_time(&self.data)
    }

    pub fn user_time(&self) -> () {
        ThreadImpl::user_time(&self.data)
    }

    pub fn create_time(&self) -> () {
        ThreadImpl::create_time(&self.data)
    }

    pub fn wait_time(&self) -> () {
        ThreadImpl::wait_time(&self.data)
    }

    pub fn context_switches(&self) -> () {
        ThreadImpl::context_switches(&self.data)
    }
}

impl Debug for Thread {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), std::fmt::Error> {
        ThreadImpl::fmt("Thread", &self.data, formatter)
    }
}

#[derive(Clone)]
pub struct ThreadS<'a> {
    data: &'a SYSTEM_THREAD_INFORMATION,
}

impl<'a> ThreadS<'a> {
    fn new(data: &'a SYSTEM_THREAD_INFORMATION) -> ThreadS<'a> {
        ThreadS { data }
    }

    pub fn thread_id(&self) -> u32 {
        ThreadImpl::thread_id(self.data)
    }

    pub fn process_id(&self) -> u32 {
        ThreadImpl::process_id(self.data)
    }

    pub fn start_address(&self) -> usize {
        ThreadImpl::start_address(self.data)
    }

    pub fn thread_state(&self) -> ThreadState {
        ThreadImpl::thread_state(self.data)
    }

    pub fn wait_reason(&self) -> ThreadWaitReason {
        ThreadImpl::wait_reason(self.data)
    }

    pub fn priority(&self) -> u32 {
        ThreadImpl::priority(self.data)
    }

    pub fn base_priority(&self) -> u32 {
        ThreadImpl::base_priority(self.data)
    }

    pub fn kernel_time(&self) -> () {
        ThreadImpl::kernel_time(self.data)
    }

    pub fn user_time(&self) -> () {
        ThreadImpl::user_time(self.data)
    }

    pub fn create_time(&self) -> () {
        ThreadImpl::create_time(self.data)
    }

    pub fn wait_time(&self) -> () {
        ThreadImpl::wait_time(self.data)
    }

    pub fn context_switches(&self) -> () {
        ThreadImpl::context_switches(self.data)
    }
}

impl Debug for ThreadS<'_> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), std::fmt::Error> {
        ThreadImpl::fmt("ThreadS", self.data, formatter)
    }
}

impl Into<Thread> for ThreadS<'_> {
    fn into(self) -> Thread {
        Thread::new(self.data.clone())
    }
}

struct ThreadImpl;

impl ThreadImpl {
    fn thread_id(x: &SYSTEM_THREAD_INFORMATION) -> u32 {
        x.ClientId.UniqueThread as u32
    }

    fn process_id(x: &SYSTEM_THREAD_INFORMATION) -> u32 {
        x.ClientId.UniqueProcess as u32
    }

    fn start_address(x: &SYSTEM_THREAD_INFORMATION) -> usize {
        x.StartAddress as usize
    }

    fn thread_state(x: &SYSTEM_THREAD_INFORMATION) -> ThreadState {
        ThreadState::from(x.ThreadState)
    }

    fn wait_reason(x: &SYSTEM_THREAD_INFORMATION) -> ThreadWaitReason {
        ThreadWaitReason::from(x.WaitReason)
    }

    fn priority(x: &SYSTEM_THREAD_INFORMATION) -> u32 {
        x.Priority as u32
    }

    fn base_priority(x: &SYSTEM_THREAD_INFORMATION) -> u32 {
        x.BasePriority as u32
    }

    fn kernel_time(_: &SYSTEM_THREAD_INFORMATION) -> () {
        ()
    }

    fn user_time(_: &SYSTEM_THREAD_INFORMATION) -> () {
        ()
    }

    fn create_time(_: &SYSTEM_THREAD_INFORMATION) -> () {
        ()
    }

    fn wait_time(_: &SYSTEM_THREAD_INFORMATION) -> () {
        ()
    }

    fn context_switches(_: &SYSTEM_THREAD_INFORMATION) -> () {
        ()
    }

    fn fmt(
        name: &str,
        x: &SYSTEM_THREAD_INFORMATION,
        formatter: &mut Formatter,
    ) -> Result<(), std::fmt::Error> {
        formatter
            .debug_struct(name)
            .field("thread_id", &ThreadImpl::thread_id(x))
            .field("process_id", &ThreadImpl::process_id(x))
            .field("start_address", &ThreadImpl::start_address(x))
            .field("thread_state", &ThreadImpl::thread_state(x))
            .field("wait_reason", &ThreadImpl::wait_reason(x))
            .field("priority", &ThreadImpl::priority(x))
            .field("base_priority", &ThreadImpl::base_priority(x))
            .field("kernel_time", &ThreadImpl::kernel_time(x))
            .field("user_time", &ThreadImpl::user_time(x))
            .field("create_time", &ThreadImpl::create_time(x))
            .field("wait_time", &ThreadImpl::wait_time(x))
            .field("context_switches", &ThreadImpl::context_switches(x))
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ThreadState {
    /// thread has been initialized, but has not started yet.
    Initialized,

    /// thread is in ready state.
    Ready,

    /// thread is running.
    Running,

    /// thread is in standby state.
    Standby,

    /// thread has exited.
    Terminated,

    /// thread is waiting.
    Wait,

    /// thread is transitioning between states.
    Transition,

    /// thread state is unknown.
    Unknown(u32),
}

impl From<u32> for ThreadState {
    fn from(value: u32) -> ThreadState {
        match value {
            0 => ThreadState::Initialized,
            1 => ThreadState::Ready,
            2 => ThreadState::Running,
            3 => ThreadState::Standby,
            4 => ThreadState::Terminated,
            5 => ThreadState::Wait,
            6 => ThreadState::Transition,
            x => ThreadState::Unknown(x),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ThreadWaitReason {
    /// thread is waiting for the scheduler.
    Executive,

    /// thread is waiting for a free virtual memory page.
    FreePage,

    /// thread is waiting for a virtual memory page to arrive in memory.
    PageIn,

    /// thread is waiting for a system allocation.
    SystemAllocation,

    /// thread execution is delayed.
    ExecutionDelay,

    /// thread execution is suspended.
    Suspended,

    /// thread is waiting for a user request.
    UserRequest,

    /// thread is waiting for event pair high.
    EventPairHigh,

    /// thread is waiting for event pair low.
    EventPairLow,

    /// thread is waiting for a local procedure call to arrive.
    LpcReceive,

    /// thread is waiting for reply to a local procedure call to arrive.
    LpcReply,

    /// thread is waiting for virtual memory.
    VirtualMemory,

    /// thread is waiting for a virtual memory page to be written to disk.
    PageOut,

    /// thread is waiting for an unknown reason.
    Unknown(u32),
}

impl From<u32> for ThreadWaitReason {
    fn from(value: u32) -> ThreadWaitReason {
        match value {
            0 | 7 => ThreadWaitReason::Executive,
            1 | 8 => ThreadWaitReason::FreePage,
            2 | 9 => ThreadWaitReason::PageIn,
            3 | 10 => ThreadWaitReason::SystemAllocation,
            4 | 11 => ThreadWaitReason::ExecutionDelay,
            5 | 12 => ThreadWaitReason::Suspended,
            6 | 13 => ThreadWaitReason::UserRequest,
            14 => ThreadWaitReason::EventPairHigh,
            15 => ThreadWaitReason::EventPairLow,
            16 => ThreadWaitReason::LpcReceive,
            17 => ThreadWaitReason::LpcReply,
            18 => ThreadWaitReason::VirtualMemory,
            19 => ThreadWaitReason::PageOut,
            x => ThreadWaitReason::Unknown(x),
        }
    }
}
