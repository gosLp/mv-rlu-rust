package javasrc.tests;

import java.util.Random;

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
        Random random = new Random();
        
        long startTime = System.nanoTime();
        for (int i = 0; i < iterations; i++)
        {
            double value = random.nextDouble() * 100;
            int key = (int)value;
            double op = random.nextDouble();
            
            if (op <= writePercent)
            {
                double op2 = random.nextDouble();
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
        setTotalTime(endTime - startTime);
    }    
}