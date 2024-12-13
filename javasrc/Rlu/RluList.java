package javasrc.Rlu;

public abstract class RluList 
{
    protected RluNode head;

    public abstract boolean add(int key);
    public abstract boolean remove(int key);
    public abstract boolean contains(int key);

    @SuppressWarnings("unused")
    private void print(Object o)
    {
        if (o == null) System.out.println("Null");
        else System.out.println(o.toString());
    }

    public void printList()
    {
        System.out.print("List:\n {");
        
        RluNode curr = head;
        while (curr != null)
        {
            System.out.print(curr.key);
            curr = curr.next;
            if (curr != null)
            {
                System.out.print(", ");
            }
        }
        System.out.println(" }\n");
    }
}
