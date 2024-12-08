public class MvRluThread extends Thread implements ThreadId
{
    public int id;
    private static int ID_GEN = 0;
    
    public MvRluLog log = null;
    public long localClock = 0;
    public long writeSetCommitTimestamp = 0;
    public boolean isWriter = false;
    public int writeSetStartIdx = 0;
    public int writeSetLength = 0;

    /*
     * =========================================
     * ThreadId Gen
     * =========================================
     */
    public MvRluThread()
    {
        id = ID_GEN++;
    }
    public int getThreadId()
    {
        return id;
    }
    public static void reset()
    {
        ID_GEN = 0;
    }
    /*
     * =========================================
     * End ThreadId Gen
     * =========================================
     */

     public void setLocalClock(long value)
     {
        localClock = value;
     }

    /*
     * =========================================
     * Log Updates
     * =========================================
     */

     public CopyObj beginLogAppend()
     {
        CopyObj result = new CopyObj();

        // TODO: Add to log
        
        return result;
     }

     public void abortLogAppend()
     {
        
     }
     public void endLogAppend()
     {

     }
     public CopyObj getLog(int index)
     {
        return log.at(index);
     }
     /*
     * =========================================
     * End Log Updates
     * =========================================
     */
}
