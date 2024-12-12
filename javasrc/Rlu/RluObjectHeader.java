package javasrc.Rlu;

import java.util.concurrent.atomic.AtomicReference;

public class RluObjectHeader<T extends RluObject>
{
    public AtomicReference<Object> ptrCopyObj;
    public WriteSetHeader<T> writeSetHeader;
}
