public class CopyObj
{
    public long commitTimestamp;
    public long olderTimestamp;
    public CopyObj older;
    private MvRluObj obj;

    public MvRluObj getObjData()
    {
        return obj;
    }
    
    public MvRluObj cloneObjData()
    {
        try
        {
            return (MvRluObj)obj.clone();
        }
        catch (Exception e)
        {
            e.printStackTrace();
        }

        return null;
    }
    public void setObjData(MvRluObj other)
    {
        obj = other;   
    }
}
