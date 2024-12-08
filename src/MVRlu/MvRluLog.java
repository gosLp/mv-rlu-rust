public class MvRluLog {
    private CopyObj[] log;
    
    private int head;
    private int tail;
    private int capacity;

    public MvRluLog(int maxLogEntries)
    {
        log = new CopyObj[maxLogEntries];
        capacity = maxLogEntries;
        head = 0;
        tail = 0;
    }

    public CopyObj at(int index)
    {
        return log[index % capacity];
    }
}
