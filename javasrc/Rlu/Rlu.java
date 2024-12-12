package javasrc.Rlu;

import java.util.concurrent.atomic.AtomicInteger;

public class Rlu
{
    private static AtomicInteger globalClock;
    private static AtomicInteger numThreads;
    private static RluThread[] threads;

    public static final int RLU_MAX_LOG_SIZE = 128;
    public static final int RLU_MAX_THREADS = 32;
    public static final int RLU_MAX_FREE_NODES = 100;

    /*
     * ===========================
     * Start public Rlu interface
     * ===========================
     */

    public static void initialize()
    {
        globalClock = new AtomicInteger(0);
        numThreads = new AtomicInteger(0);
        threads = new RluThread[RLU_MAX_THREADS];

        for (int i = 0; i < RLU_MAX_THREADS; i++)
        {
            threads[i] = null;
        }
    }
    
    public static void initializeThread()
    {
        int id = numThreads.getAndIncrement();
        
        if (id >= RLU_MAX_THREADS)
        {
            System.err.println("Created too many threads!");
            return;
        }
        
        RluThread thread = currentThread();
        thread.setId(id);
        threads[id] = thread;
    }

    public static void readerLock()
    {
        RluThread thread = currentThread();
        assert (thread.runCounter.get() & 0x1) != 0 : "Thread's run counter was odd on readerLock, indicating that it was already in the critical section";

        thread.runCounter.getAndIncrement();
        thread.isWriter = false;
        thread.localClock.set(globalClock.get()); 
    }

    public static void readerUnlock()
    {
        RluThread thread = currentThread();
        assert (thread.runCounter.get() & 0x1) != 0 : "Thread's run counter was even on readerUnlock, indicating that it was not in the critical section";

        thread.runCounter.getAndIncrement();
        if (thread.isWriter)
        {
            thread.isWriter = false;
            commitWriteLog();
        }
    }

    public static RluObject getReference(RluObject object)
    {
        // Do not process null inputs
        if (object == null) return object;

        RluObject ptrCopy = object.getPtrObjCopy();
        
        // Object is unlocked
        if (ptrCopy == null) return object;
        
        // This object is a copy and has already been referenced
        if (object.isCopy()) return object;

        // Get locking thread id
        int lockingThreadId = ptrCopy.getLockingThreadIdFromWriteSet();
        if (lockingThreadId >= RLU_MAX_THREADS)
        {
            System.err.println("The thread id in a writeset was invalid");
            return object;
        }

        // Locked by us
        RluThread thread = currentThread();
        if (lockingThreadId == thread.getThreadId())
        {
            return ptrCopy;
        }

        // Check for stealing
        int myLocalClock = thread.localClock.get();
        int otherWriteClock = threads[lockingThreadId].writeClock.get();
        
        // Steal
        if (otherWriteClock <= myLocalClock)
        {
            return ptrCopy;
        }

        // Don't steal
        return object;
    }

    /*
     * Returns null on failure, pointer to locked object on success
     */
    public static RluObject tryLock(RluObject object)
    {
        boolean isNull = object == null;
        assert !isNull : "Object being locked was null";
        if (isNull) return null;

        RluThread thread = currentThread();

        thread.isWriter = true;
        RluObject ptrCopy = object.getPtrObjCopy();
        
        // If we are trying to lock a copy...
        if (object.isCopy())
        {
            object = object.getPtrOriginal();
        }

        // If the object is already locked...
        if (ptrCopy != null)
        {
            int lockingThreadId = ptrCopy.getLockingThreadIdFromWriteSet();

            if (lockingThreadId == thread.getThreadId())
            {
                // Check run counter to see if locked by current execution of this thread
                if (ptrCopy.getWriteSetRunCounter() == thread.runCounter.get())
                {
                    return ptrCopy;
                }
                
                // Locked by other execution of this thread
                return null;
            }

            // Locked by another thread
            return null;
        }

        // Object is unlocked
        RluObject copy = object.getCopyWithWriteSetHeader(thread.runCounter.get(), thread.getThreadId());
        thread.writeLog[thread.currPos] = copy;
        
        if (!object.cas(copy)) 
        {
            return null;
        }
        // Update write set header
        thread.currPos++;
        thread.numObjs++;
        
        return copy;
    }

    public static void abort()
    {
        RluThread thread = currentThread();

        int prevRunCounter = thread.runCounter.getAndIncrement();
        assert (prevRunCounter & 0x1) != 0 : "Run counter was even during abort indicating thread was not in critical section";
        
        if (thread.isWriter)
        {
            thread.isWriter = false;
            unlockObjects();
        }
    }

    public static void rluFree(RluObject object)
    {
        assert object.isCopy() : "Can't free a node you haven't locked";
        currentThread().addToFree(object);
    }

    public static RluObject rluGetPtr(RluObject object)
    {
        if (object == null) return null;
        if (object.isCopy()) return object.getPtrOriginal();
        return object;
    }

    /*
     * ===========================
     * End public Rlu interface
     * ===========================
     */
    
    
    public static void processFree()
    {
        currentThread().clearFreeList();
    }

    public static void synchronize()
    {
        RluThread thread = currentThread();
        
        // Collect all threads we must wait for
        for (int i = 0; i < RLU_MAX_THREADS; i++)
        {
            // Don't wait on self or null threads
            if (threads[i] == null || i == thread.getThreadId()) continue;

            thread.waitOnThreads[i].runCounter = threads[i].runCounter.get();
            
            // Thread is still in critical section, must wait on it
            thread.waitOnThreads[i].isWait = (thread.waitOnThreads[i].runCounter & 0x1) == 0x1;
        }

        for (int i = 0; i < RLU_MAX_THREADS; i++)
        {
            while (true)
            {
                // Don't wait on threads which do not require it
                if (!thread.waitOnThreads[i].isWait) break;
                
                RluThread other = threads[i];
                
                // Check if other thread has progressed past critical section
                if (thread.waitOnThreads[i].runCounter != other.runCounter.get()) break;

                // Don't wait on threads that started after this one
                if (thread.writeClock.get() <= other.localClock.get()) break;
            }
        }
    }

    public static void swapWriteLogs()
    {
        RluThread thread = currentThread();
        
        // Two write logs are stored as either half of one array. To swap them, clear the current one
        // and set current position to the other
        if (thread.currPos < (RLU_MAX_LOG_SIZE / 2))
        {
            for (int i = RLU_MAX_LOG_SIZE / 2; i < RLU_MAX_LOG_SIZE; i++)
            {
                thread.writeLog[i] = null;
            }
            // Swaps write logs
            thread.currPos = RLU_MAX_LOG_SIZE / 2;
        }
        else 
        {
            for (int i = 0; i < RLU_MAX_LOG_SIZE / 2; i++)
            {
                thread.writeLog[i] = null;
            }
            // Swaps write logs
            thread.currPos = 0;
        }
    }
    
    private static void commitWriteLog()
    {
        RluThread thread = currentThread();

        thread.writeClock.set(globalClock.get() + 1);
        globalClock.getAndIncrement();

        synchronize();
        writebackWriteLog();

        thread.writeClock.set(Integer.MAX_VALUE);

        swapWriteLogs();
        processFree();
    }
    
    private static void writebackWriteLog()
    {
        RluThread thread = currentThread();
        for (int i = thread.currPos - thread.numObjs; i < thread.currPos; i++)
        {
            assert thread.writeLog[i] != null : "An entry in the write log was null";
            thread.writeLog[i].copyBackToOriginal();
        }

        thread.numObjs = 0;
    }

    private static void unlockObjects()
    {
        RluThread thread = currentThread();

        for (int i = thread.currPos - thread.numObjs; i < thread.currPos; i++)
        {
            assert thread.writeLog[i] != null : "There was a null value in the write log";
            thread.writeLog[i].unlockOriginal();
        }

        thread.currPos -= thread.numObjs;
        thread.numObjs = 0;
    }
    
    private static RluThread currentThread()
    {
        return (RluThread)Thread.currentThread();
    }

    @SuppressWarnings("unused")
    private static void print(Object o)
    {
        if (o == null) System.out.println("Null");
        else System.out.println(o.toString());
    }
}
