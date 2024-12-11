package java;

import java.util.concurrent.atomic.AtomicReference;

public class RluObjectHeader<T extends RluObject>
{
    public AtomicReference<T> ptrCopyObj;
    public WriteSetHeader<T> writeSetHeader;
}
