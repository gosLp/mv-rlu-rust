const RLU_MAX_LOG_SIZE: usize = 128;
const RLU_MAX_THREADS: usize = 32;
const RLU_MAX_FREE_NODES: usize = 100;
pub const PTR_ID_OBJ_COPY: usize = 0x12341234;

pub struct RluGlobal 
{
    threads : [RluThread, RLU_MAX_THREADS],
    global_clock : AtomicU64,
    num_threads_created : AtomicUsize
} 

pub fn mvrlu_alloc()
{
    // Allocate a new master object with the correct header
}

pub fn mvrlu_free()
{
    // Object must already be locked by mvrlu_try_lock()
    // Add object to free list
}
pub fn mvrlu_read_lock()
{
    // Increment run counter
    // Update local clock
}
pub fn mvrlu_read_unlock()
{
    // Increment run counter

    /*
     * If is writer:
     * Move pending updates to head of copy list
     * Update write set commit timestamp
     * Update commit timestamp of all objects in write set and set pending = NULL
     */
}

pub fn mvrlu_dereference()
{
    // Traverse version history and return the first object whose commit timestamp is less than the local clock. 
    // If no such object exists, return the master object.
}

pub fn mvrlu_try_lock()
{
    // if p_pending is not null, abort and retry 
    // if the writer's local clock is less than the commit timestamp of the object, abort and retry

    // Create header in log and try to install into pending using CAS. If unsuccessful, remove header from log and abort and retry
    // Add object to write set
}

pub fn abort_write()
{
    // Set pending to NULL
    // Remove header from log
}