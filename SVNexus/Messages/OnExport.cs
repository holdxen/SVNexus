using CommunityToolkit.Mvvm.Messaging.Messages;
using SVNexus.Generated;

namespace SVNexus.Messages;

public class OnExport(ExportOptions value) : ValueChangedMessage<ExportOptions>(value);