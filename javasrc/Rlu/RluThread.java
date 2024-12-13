package javasrc.Rlu;


public class RluThread extends Thread 
{
    private int id;
    public int localClock;
    public int writeClock;
    public int runCounter;
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

    protected long totalRluNanoTime = 0;
    protected long generalNanoTime = 0;
    protected long totalTime = 0;

    public RluThread()
    {
        isWriter = false;
        runCounter = 0;
        localClock = 0;
        writeClock = Integer.MAX_VALUE;
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

    public long getTotalRluNanoTime() 
    {
        return totalRluNanoTime;
    }
    public void addRluNanoTime(long time)
    {
        totalRluNanoTime += time;
    }
    public long getGeneralNanoTime() 
    {
        return generalNanoTime;
    }
    public void addGeneralNanoTime(long time)
    {
        generalNanoTime += time;
    }
    public long getTotalTime() 
    {
        return totalTime;
    }
    public void setTotalTime(long time)
    {
        totalTime = time;
    }
}
