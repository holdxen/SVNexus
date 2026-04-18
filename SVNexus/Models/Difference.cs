using System.Collections.Generic;

namespace SVNexus.Models;

public class Difference
{
    public List<DifferenceLine> Original { get; set; } = [];
    
    public List<DifferenceLine> Modified { get; set; } = [];
    
}