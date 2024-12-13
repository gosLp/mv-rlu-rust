package javasrc.Rlu;


/*
 * THERE IS SOME BUG WITH THIS CLASS OR WITH RLU IMPLEMENTATION THAT HAS TO DO WITH CASTING BETWEEN RLUOBJECT AND OBJECT.
 *      This is likely a problem due to how the unique copy pointer is defined as an object since the RluObject type is abstract.
 *      If there is time, this fix would likely improve the write performance since it falls off very quickly
 */
public class RluFineList extends RluList
{
    public RluFineList()
    {
        head = new RluNode(Integer.MIN_VALUE);
        head.next = new RluNode(Integer.MAX_VALUE);
    }

    public boolean contains(int key)
    {
        Rlu.readerLock();
        RluNode curr = (RluNode)Rlu.getReference(head);

        while (curr != null && curr.key <= key)
        {
            if (curr.key == key) return true;
            curr = (RluNode)Rlu.getReference(curr.next);
        }
        
        Rlu.readerUnlock();
        return false;
    }
    public boolean add(int key)
    {
        RluNode prev = null, curr = null;
        try
        {
            while (true)
            {
                Rlu.readerLock();

                prev = (RluNode)Rlu.getReference(head);
                curr = (RluNode)Rlu.getReference(prev.next);

                while (curr.key < key)
                {
                    prev.unlock();
                    prev = curr;
                    curr = (RluNode)Rlu.getReference(curr.next);
                    curr.lock();                    
                }

                if (curr == null || curr.key == key) return false;

                RluNode lockedPrev = (RluNode)Rlu.tryLock(prev);
                if (lockedPrev == null)
                {
                    Rlu.abort();
                    continue;
                }
                
                if (curr != null)
                {
                    RluNode lockedCurr = (RluNode)Rlu.tryLock(curr);
                    if (lockedCurr == null)
                    {
                        Rlu.abort();
                        continue;
                    }
                }
                
                RluNode newNode = new RluNode(key);
                newNode.next = (RluNode)Rlu.rluGetPtr(curr);
                prev.next = (RluNode)Rlu.rluGetPtr(newNode);
                return true;
            }
        }
        finally {
            if (!(prev instanceof Object) && !prev.isCopy())
            {
                prev.unlock();
            }
            if (!(curr instanceof Object) && !curr.isCopy())
            {
                curr.unlock();
            }
            Rlu.readerUnlock();
        }
    }
    public boolean remove(int key)
    {
        RluNode prev = null, curr = null;
        try
        {
            while (true)
            {
                Rlu.readerLock();

                prev = (RluNode)Rlu.getReference(head);
                prev.lock();
                curr = (RluNode)Rlu.getReference(prev.next);
                curr.lock();

                while (curr != null && curr.key < key)
                {
                    prev.unlock();
                    prev = curr;
                    curr = (RluNode)Rlu.getReference(curr.next);
                    curr.lock();
                }

                if (curr == null || curr.key > key) return false;

                RluNode lockedPrev = (RluNode)Rlu.tryLock(prev);
                if (lockedPrev == null)
                {
                    Rlu.abort();
                    continue;
                }
                
                if (curr != null)
                {
                    RluNode lockedCurr = (RluNode)Rlu.tryLock(curr);
                    if (lockedCurr == null)
                    {
                        Rlu.abort();
                        continue;
                    }
                    if (curr.next != null)
                    {
                        RluNode lockedCurrNext = (RluNode)Rlu.tryLock(curr.next);
                        if (lockedCurrNext == null)
                        {
                            Rlu.abort();
                            continue;
                        }
                    }
                }

                prev.next = (RluNode)Rlu.rluGetPtr(curr.next);
                return true;
            }
        }
        finally {
            prev.unlock();
            curr.unlock();
            Rlu.readerUnlock();
        }
    }
}