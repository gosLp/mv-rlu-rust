package java;

public abstract class RluObject<T extends RluObject<T>>
{
    private static final class CopyClass extends RluObject<CopyClass> 
    {
        @Override
        public CopyClass getCopyWithWriteSetHeader(int __, int ___)
        {
            return null;
        }
        @Override
        public void copyBackToOriginal() {}
    }
    public final RluObject<CopyClass> COPYID = new CopyClass();
    
    protected RluObjectHeader<T> header;
    
    public RluObject<T> getPtrObjCopy()
    {
        return null;
    }

    public boolean isLocked()
    {
        return getPtrObjCopy() != null;
    }

    public boolean isCopy()
    {
        return getPtrObjCopy() == COPYID;
    }

    
    public T getPtrOriginal()
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

    public abstract T getCopyWithWriteSetHeader(int runCounter, int getThreadId);
    
    public boolean cas(T newObject)
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
}
