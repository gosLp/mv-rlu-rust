package javasrc.tests;

import java.lang.Math;
import javasrc.Rlu.RluList;
import javasrc.Rlu.RluThread;

public class ListThread extends RluThread 
{
    private RluList list;
    private double writePercent;
    private int iterations;

    public ListThread(RluList list, double writePercent, int iterations)
    {
        this.list = list;
        this.iterations = iterations;
        this.writePercent = writePercent;
    }
    @Override 
    public void run()
    {
        long startTime = System.nanoTime();

        for (int i = 0; i < iterations; i++)
        {
            double value = Math.random() * 100;
            int key = (int)value;
            
            double op = Math.random();
            if (op <= writePercent)
            {
                double op2 = Math.random();
                if (op2 < 0.5)
                {
                    list.remove(key);
                }
                else 
                {
                    list.add(key);
                }
            }
            else 
            {
                list.contains(key);
            }
        }

        long endTime = System.nanoTime();
        totalTimeNano = endTime - startTime;
    }    
}