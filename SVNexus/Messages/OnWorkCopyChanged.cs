namespace SVNexus.Messages;

public class OnWorkCopyChanged
{
    public enum Action
    {
        Modify,
        Add,
        Delete
    }
    
    public required string Path { get; set; }
    
    public object? Sender { get; set; }
}