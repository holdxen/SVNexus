using System;
using System.Collections.Generic;
using SVNexus.Components;

namespace SVNexus.Extension;

public static class DifferenceLineExtension
{
    extension(List<DifferenceLine> differences)
    {
        public int ExcludeIndexToRealIndex(int index, DifferenceLine.Kind[] exclude)
        {

            var excludeIndex = 0;
            var realInex = 0;

            foreach (var line in differences)
            {
                if (exclude.Contains(line.DifferenceKind))
                {
                    realInex++;
                    continue;
                }

                if (excludeIndex == index)
                {
                    return realInex;
                }
                
                excludeIndex++;
                realInex++;
            }
            
            return -1;
        }
    }
}