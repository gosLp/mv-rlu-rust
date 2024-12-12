package javasrc.Rlu;

import java.util.concurrent.atomic.AtomicReference;

public abstract class RluObject
{
    public static final Object COPYID = new Object();
    
    protected RluObjectHeader<RluObject> header;
    
    public RluObject()
    {
        header = new RluObjectHeader<>();
        header.ptrCopyObj = new AtomicReference<>(null);
    }

    public RluObject getPtrObjCopy()
    {
        return (RluObject)header.ptrCopyObj.get();
    }

    public boolean isLocked()
    {
        return getPtrObjCopy() != null;
    }

    public boolean isCopy()
    {
        return getPtrObjCopy() == COPYID;
    }

    
    public RluObject getPtrOriginal()
    {
        return header.writeSetHeader.ptrActualObject;
    }

    public int getLockingThreadIdFromWriteSet()
    {
        return header.writeSetHeader.threadId;
    }

    public int getWriteSetRunCounter()
    {
        return header.writeSetHeader.runCounter;
    }

    public abstract RluObject getCopyWithWriteSetHeader(int runCounter, int getThreadId);
    
    public boolean cas(RluObject newObject)
    {
        return header.ptrCopyObj.compareAndSet(null, newObject);
    }


    public abstract void copyBackToOriginal();

    public void unlockOriginal()
    {
        getPtrOriginal().unlock();
    }
    public void unlock()
    {
        header.ptrCopyObj.set(null);
    }

    @SuppressWarnings("unused")
    private void print(Object o)
    {
        if (o == null) System.out.println("Null");
        else System.out.println(o.toString());
    }
}
