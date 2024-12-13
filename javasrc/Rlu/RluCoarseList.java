package javasrc.Rlu;

import java.util.concurrent.locks.ReentrantLock;

public class RluCoarseList extends RluList
{
    ReentrantLock lock = new ReentrantLock();
    
    public RluCoarseList()
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
        try
        {
            while (true)
            {
                lock.lock();
                Rlu.readerLock();

                RluNode prev = (RluNode)Rlu.getReference(head);
                RluNode curr = (RluNode)Rlu.getReference(prev.next);

                while (curr != null && curr.key < key)
                {
                    prev = curr;
                    curr = (RluNode)Rlu.getReference(curr.next);
                }

                if (curr != null && curr.key == key) return false;

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
            Rlu.readerUnlock();
            lock.unlock();
        }
    }
    public boolean remove(int key)
    {
        try
        {
            while (true)
            {
                lock.lock();
                Rlu.readerLock();

                RluNode prev = (RluNode)Rlu.getReference(head);
                RluNode curr = (RluNode)Rlu.getReference(prev.next);

                while (curr != null && curr.key < key)
                {
                    prev = curr;
                    curr = (RluNode)Rlu.getReference(curr.next);
                }

                if (curr != null && curr.key < key) return false;

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
                else 
                {
                    prev.next = null;
                    return true;
                }

                prev.next = (RluNode)Rlu.rluGetPtr(curr.next);
                return true;
            }
        }
        finally {
            Rlu.readerUnlock();
            lock.unlock();
        }
    }

    @SuppressWarnings("unused")
    private void print(Object o)
    {
        if (o == null) System.out.println("Null");
        else System.out.println(o.toString());
    }

    public void printList()
    {
        System.out.print("List:\n {");
        
        RluNode curr = head;
        while (curr != null)
        {
            System.out.print(curr.key);
            curr = curr.next;
            if (curr != null)
            {
                System.out.print(", ");
            }
        }
        System.out.println(" }\n");
    }
}
