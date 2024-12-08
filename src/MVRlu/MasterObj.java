import java.util.concurrent.atomic.AtomicReference;

public class MasterObj 
{
    public CopyObj versionChain;
    public AtomicReference<CopyObj> pending;
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
}

