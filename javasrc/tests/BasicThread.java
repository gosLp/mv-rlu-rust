package javasrc.tests;

import javasrc.Rlu.Rlu;
import javasrc.Rlu.RluThread;

public class BasicThread extends RluThread 
{
    @Override
    public void run()
    {
        Rlu.initializeThread();
        print("Running thread " + getThreadId());        
    }
}