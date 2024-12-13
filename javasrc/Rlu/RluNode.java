package javasrc.Rlu;

import java.util.concurrent.atomic.AtomicReference;
import java.util.concurrent.locks.ReentrantLock;

public class RluNode extends RluObject 
{
    public RluNode next;
    public int key;

    private ReentrantLock lock = new ReentrantLock();

    public RluNode(int key)
    {
        super();
        this.key = key;
    }

    @Override
    public RluNode getCopyWithWriteSetHeader(int runCounter, int threadId) 
    {
        RluObjectHeader<RluObject> newHeader = new RluObjectHeader<>();
        newHeader.ptrCopyObj = new AtomicReference<Object>(RluObject.COPYID);
        
        newHeader.writeSetHeader = new WriteSetHeader<RluObject>();
        newHeader.writeSetHeader.threadId = threadId;
        newHeader.writeSetHeader.runCounter = runCounter;
        newHeader.writeSetHeader.ptrActualObject = this;

        RluNode newNode = new RluNode(key);
        newNode.header = newHeader;
        newNode.next = next;

        return newNode;
    }

    @Override
    public void copyBackToOriginal() 
    {
        //((RluNode)(header.writeSetHeader.ptrActualObject)).next = next;
        //((RluNode)(header.writeSetHeader.ptrActualObject)).key = key;
        header.writeSetHeader.ptrActualObject.header.ptrCopyObj.set(null);
    }

    public void lock()
    {
        lock.lock();
    }
    public void unlock()
    {
        lock.unlock();
    }
}
