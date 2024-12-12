package javasrc.tests;

import java.util.Arrays;

import javasrc.Rlu.Rlu;
import javasrc.Rlu.RluList;
import javasrc.Rlu.RluThread;

public class Bench 
{
    private static final String BASIC = "basic";
    private static final String LIST = "list";

    public static void main(String args[])
    {
        int numThreads = args.length > 0 ? Integer.parseInt(args[0]) : 2;
        String test = (args.length > 1 ? args[1] : BASIC).toLowerCase();
        double percentWrite = args.length > 2 ? Double.parseDouble(args[2]) : 0.1f;
        int iterations = args.length > 3 ? Integer.parseInt(args[3]) : 1000000;

        if (!validateTest(test))
        {
            System.out.println("Basic test being run because " + test + " is not an option");
            test = BASIC;
        }

        Rlu.initialize();
        runBenchmark(numThreads, test, percentWrite, iterations);        
    }
    private static boolean validateTest(String name)
    {
        return name.equals(BASIC) || name.equals(LIST);
    }

    private static void runBenchmark(int numThreads, String test, double percentWrite, int iterations)
    {
        print("=====================");
        print("Test: " + test + "\nThread count: " + numThreads);
        print("=====================\n");

        RluList list = new RluList();

        RluThread[] threads = getTestThreads(numThreads, test, list, percentWrite, iterations);
        
        long startTime = System.nanoTime();
        Arrays.stream(threads).forEach(thread -> thread.start());
        Arrays.stream(threads).forEach(thread -> {
            try { thread.join(); }
            catch (Exception e) { e.printStackTrace(); }
        });
        long endTime = System.nanoTime();

        if (test.equals(LIST)) list.printList();
        double totalIterations = numThreads * iterations;
        double totalTime = (double)(endTime - startTime) / 1000000000;
        print("Total operations: " + (long)totalIterations);
        print("Total time: " + totalTime + "s");
        print("Throughput = " + totalIterations / totalTime);
    }

    private static RluThread[] getTestThreads(int num, String test, RluList list, double writePercent, int iterations)
    {
        RluThread[] threads = new RluThread[num];

        for (int i = 0; i < num; i++)
        {
            if (test.equals(LIST)) threads[i] = new ListThread(list, writePercent, iterations);
            else threads[i] = new BasicThread();
        }

        return threads;
    }
    private static void print(Object obj)
    {
        System.out.println(obj.toString());
    }
}