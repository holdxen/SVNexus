using System.Threading.Tasks;
using CommunityToolkit.Mvvm.Input;

namespace SVNexus.Extension;

public static class RelayCommandExtension
{
    extension(IAsyncRelayCommand command)
    {
        public void ExecuteOrNothing(object? parameter)
        {
            if (command.CanExecute(parameter))
            {
                command.Execute(parameter);
            }
        }

        public async Task ExecuteOrNothingAsync(object? parameter)
        {
            if (command.CanExecute(parameter))
            {
                await command.ExecuteAsync(parameter);
            }
        }
    }
}