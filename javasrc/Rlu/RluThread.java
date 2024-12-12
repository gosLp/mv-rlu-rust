package javasrc.Rlu;

import java.util.concurrent.atomic.AtomicInteger;

public class RluThread extends Thread 
{
    private int id;
    public AtomicInteger localClock;
    public AtomicInteger writeClock;
    public AtomicInteger runCounter;
    public boolean isWriter;

    // For tracking when we can overwrite data in log
    private RluObject[] freeNodes;
    private int freeNodesSize = 0;

    // For syncing
    public WaitEntry[] waitOnThreads;

    // Log state
    public RluObject[] writeLog;
    public int currPos = 0;
    public int numObjs = 0;

    protected long totalTimeNano;

    public RluThread()
    {
        isWriter = false;
        runCounter = new AtomicInteger(0);
        localClock = new AtomicInteger(0);
        writeClock = new AtomicInteger(Integer.MAX_VALUE);
        writeLog = new RluObject[Rlu.RLU_MAX_LOG_SIZE];
        freeNodes = new RluObject[Rlu.RLU_MAX_FREE_NODES];
        waitOnThreads = new WaitEntry[Rlu.RLU_MAX_THREADS];

        for (int i = 0; i < Rlu.RLU_MAX_THREADS; i++)
        {
            waitOnThreads[i] = new WaitEntry();
        }
        for (int i = 0; i < Rlu.RLU_MAX_FREE_NODES; i++)
        {
            freeNodes[i] = null;
        }   
    }

    public int getThreadId()
    {
        return id;
    }
    public void setId(int newId)
    {
        id = newId;
    }

    public void addToFree(RluObject object)
    {
        freeNodes[freeNodesSize] = object.getPtrOriginal();
        freeNodesSize++;
    }
    
    public void clearFreeList()
    {
        for (int i = 0; i < freeNodesSize; i++)
        {
            freeNodes[i] = null;
        }
    }

    public void print(Object obj)
    {
        if (obj == null)
        {
            System.out.println("Null");
        }
        else 
        {
            System.out.println(obj.toString());
        }
    }

    public long getTotalTimeNano() 
    {
        return totalTimeNano;
    }
}
