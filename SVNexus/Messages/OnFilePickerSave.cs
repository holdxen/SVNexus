using Avalonia.Platform.Storage;
using CommunityToolkit.Mvvm.Messaging.Messages;

namespace SVNexus.Messages;

public class OnFilePickerSave: AsyncRequestMessage<IStorageFile?>
{
    public required FilePickerSaveOptions Options { get; set; }
}