using System.Collections.Generic;
using Avalonia.Platform.Storage;
using CommunityToolkit.Mvvm.Messaging.Messages;

namespace SVNexus.Messages;

public class OnFolderPickerOpen(FolderPickerOpenOptions options) : AsyncRequestMessage<IReadOnlyList<IStorageFolder>>
{
    public FolderPickerOpenOptions Options { get; } =  options;
}