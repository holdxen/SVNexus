using System;
using System.Collections.Generic;
using System.Linq;
using SVNexus.Components;
using SVNexus.Models;

namespace SVNexus.Extension;

public static class DifferenceLineExtension
{
    extension(List<DifferenceLine> differences)
    {
        public int RealIndex(int index)
        {
            if (index == 0)
            {
                return differences.TakeWhile(line => line.Content is not null).Count();
            }
            
            var excludeIndex = 0;
            var realInex = 0;
            
            foreach (var line in differences)
            {
                if (line.Content is null)
                {
                    realInex++;
                    continue;
                }
                
                excludeIndex++;
                realInex++;
            
                if (excludeIndex == index)
                {
                    return realInex;
                }
                
            }
            
            return -1;
        }
        
        public int ExcludeIndexToRealIndex(int index, DifferenceLine.Kind[] exclude)
        {
            if (index == 0)
            {
                return differences.TakeWhile(line => exclude.Contains(line.DifferenceKind)).Count();
            }

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