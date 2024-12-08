import java.util.concurrent.atomic.AtomicInteger;

public class MvRlu 
{
    public static AtomicInteger globalClock;
    public static long NUM_THREADS = 0;
    public long initializeThread()
    {
        return NUM_THREADS++;
    }
    public static void registerObj(MvRluObj data)
    {
        MasterObj obj = new MasterObj();

        obj.pending = null;
        obj.versionChain = null;
        data.setMaster(obj);        
    }
    public static void readerLock() 
    {
        MvRluThread thread = currentThread();
        thread.setLocalClock(globalClock.get());
    }
    public static void readerUnlock()
    {
        MvRluThread thread = currentThread();
        if (!thread.isWriter) return;
        
        // For each pending object, swing it to the head of the associated version chain list
        for (int i = 0; i < thread.writeSetLength; i++)
        {
            // TODO: Make sure that master is set when copyObjs are initialized
            CopyObj pending = thread.getLog(thread.writeSetStartIdx + i);
            MasterObj master = pending.getObjData().getMaster();
            CopyObj oldHead = master.versionChain;
            master.versionChain = pending;
            pending.older = oldHead;
        }

        // TODO: May not be the correct place to advance clock, but should be done in this method as part of commit
        // Update the thread's writeset ts BEFORE the commit ts so that updates are atomic 
        thread.writeSetCommitTimestamp = globalClock.incrementAndGet();

        // Update each committed object's timestamp and unlock objects by setting pending to null
        for (int i = 0; i < thread.writeSetLength; i++)
        {
            CopyObj pending = thread.getLog(thread.writeSetStartIdx + i);
            pending.commitTimestamp = globalClock.get();
            MasterObj master = pending.getObjData().getMaster();
            master.pending = null;
        }
    }
    public static CopyObj tryLock(MvRluObj obj)
    {
        MasterObj master = obj.getMaster();
        if (master.pending != null)
        {
            return null;
        }

        CopyObj oldCopy = obj.getCopy();
        MvRluThread thread = currentThread();
        if (oldCopy != null)
        {
            if (thread.localClock < oldCopy.commitTimestamp)
            {
                return null;
            }
        }

        CopyObj newCopy = thread.beginLogAppend();

        if (!tryLockObj(master, oldCopy, newCopy))
        {
            thread.abortLogAppend();
            return null;
        }

        if (oldCopy != null)
        {
            newCopy.setObjData(oldCopy.cloneObjData());
        }
        else 
        {
            newCopy.setObjData(master.cloneObjData());
        }

        thread.endLogAppend();

        thread.isWriter = true;

        // TODO: Add object to write set here?
        return newCopy;
    }
    public static MvRluObj dereference(MvRluObj data)
    {
        MasterObj master = data.getMaster();

        CopyObj versionChain = master.versionChain;
        MvRluThread thread = currentThread();

        // Search for the first copy with a timestamp from before we began the search. If one does not exist, return the master
        while (versionChain != null)
        {
            if (versionChain.commitTimestamp < thread.localClock) return versionChain.getObjData();
            
            versionChain = versionChain.older;
        }
        return master.getObjData();
    }

    private static boolean tryLockObj(MasterObj obj, CopyObj oldCopy, CopyObj newCopy)
    {
        if (obj.pending.get() != null) return false;
        if (obj.versionChain != oldCopy) return false;
        if (!obj.pending.compareAndSet(null, newCopy)) return false;

        return true;
    }
    private static void abort()
    {
        // TODO: Not sure what to do here
    }

    private static MvRluThread currentThread()
    {
        return (MvRluThread)Thread.currentThread();
    }
}
