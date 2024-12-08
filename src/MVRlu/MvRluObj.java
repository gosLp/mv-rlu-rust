public abstract class MvRluObj implements Cloneable
{
    /*
     * All client data types must inherit from this type so that object creation and management can be automated
     */    
    private MasterObj master;
    private CopyObj copy;

    public final MasterObj getMaster()
    {
        return master;
    }
    public final CopyObj getCopy()
    {
        return copy;
    }
    public final void setMaster(MasterObj newMaster)
    {
        master = newMaster;
    }
    public final void setCopy(CopyObj newCopy)
    {
        copy = newCopy;
    }

    @Override
    public Object clone() throws CloneNotSupportedException 
    {
        return super.clone();
    }
}
